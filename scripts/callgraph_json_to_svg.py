#!/usr/bin/env python3
import html
import json
import sys
from collections import defaultdict
from pathlib import Path


NODE_WIDTH = 300
NODE_HEIGHT = 44
X_GAP = 540
Y_GAP = 32
MARGIN_X = 36
MARGIN_Y = 122
EXCLUDED_SRC_FILES = {
    "shogi_legality_lite/src/mate_solver.rs",
    "shogi_legality_lite/src/perft.rs",
}


def truncate(value, limit):
    return value if len(value) <= limit else value[: limit - 1] + "..."


def node_sort_key(node):
    return (
        node["file"],
        node.get("line") or 0,
        node["qualified_path"],
    )


def group_sort_key(group):
    order = {"lib.rs": 0, "prelegality.rs": 1, "normal.rs": 2}
    return (order.get(Path(group[0]).name, 3), group[0])


def node_color(node):
    if node.get("is_cycle"):
        return "#fee2e2", "#dc2626"
    if node["file"].endswith("/src/lib.rs"):
        return "#dbeafe", "#1d4ed8"
    if "/src/" in node["file"]:
        return "#dcfce7", "#15803d"
    if "/benches/" in node["file"]:
        return "#fef3c7", "#92400e"
    return "#f8fafc", "#475569"


def text(value):
    return html.escape(str(value), quote=True)


def is_crate_internal(node):
    return node.get("is_workspace") and "/src/" in node["file"] and node["file"] not in EXCLUDED_SRC_FILES


def count_text(count):
    return f"{count} node" if count == 1 else f"{count} nodes"


def cycle_count_text(count):
    return f"{count} cycle" if count == 1 else f"{count} cycles"


def render_node(node, x, y):
    fill, stroke = node_color(node)
    name = truncate(node["qualified_path"], 40)
    location = truncate(f"{node['file']}:{node.get('line', '')}", 46)
    return f"""  <g id="{text(node['id'])}">
    <title>{text(node['id'])}</title>
    <rect x="{x}" y="{y}" width="{NODE_WIDTH}" height="{NODE_HEIGHT}" rx="6" fill="{fill}" stroke="{stroke}" stroke-width="1"/>
    <text x="{x + 12}" y="{y + 18}" fill="#0f172a" font-size="12" font-weight="600">{text(name)}</text>
    <text x="{x + 12}" y="{y + 34}" fill="#64748b" font-size="10">{text(location)}</text>
  </g>"""


def render_edge(source, target, positions, is_cycle_edge, index, ports, bottom):
    source_x, source_y = positions[source]
    target_x, target_y = positions[target]
    start_y = source_y + ports[(index, "source")]
    end_y = target_y + ports[(index, "target")]
    color = "#dc2626" if is_cycle_edge else "#475569"
    lane = 24 + index * 8
    # Long edges travel below all nodes; no path crosses an intervening column.
    if abs(source_x - target_x) > NODE_WIDTH + X_GAP:
        start_x, end_x = source_x + NODE_WIDTH, target_x + NODE_WIDTH
        rail_y = bottom + 24 + index * 10
        path = (f"M {start_x} {start_y} H {start_x + lane} V {rail_y} "
                f"H {end_x + lane} V {end_y} H {end_x}")
    elif source_x == target_x:
        start_x = end_x = source_x + NODE_WIDTH
        path = f"M {start_x} {start_y} H {start_x + lane} V {end_y} H {end_x}"
    else:
        start_x = source_x + NODE_WIDTH if source_x < target_x else source_x
        end_x = target_x if source_x < target_x else target_x + NODE_WIDTH
        lane_x = min(source_x, target_x) + NODE_WIDTH + lane
        path = f"M {start_x} {start_y} H {lane_x} V {end_y} H {end_x}"
    marker = "cycle-arrow" if is_cycle_edge else "arrow"
    return (
        f'<g class="edge" tabindex="0"><title>{text(source)} → {text(target)}</title>'
        f'<path d="{path}" fill="none" stroke="white" stroke-width="6"/>'
        f'<path class="wire" d="{path}" fill="none" stroke="{color}" '
        f'stroke-width="2" marker-end="url(#{marker})"/>'
        f'<path d="{path}" fill="none" stroke="transparent" stroke-width="10"/></g>'
    )


def find_cycle_components(node_ids, edges):
    graph = {node_id: [] for node_id in node_ids}
    self_loops = set()
    for edge in edges:
        source = edge["source"]
        target = edge["target"]
        graph[source].append(target)
        if source == target:
            self_loops.add(source)

    index = 0
    stack = []
    stacked = set()
    indexes = {}
    lowlinks = {}
    components = []

    def visit(node_id):
        nonlocal index
        indexes[node_id] = index
        lowlinks[node_id] = index
        index += 1
        stack.append(node_id)
        stacked.add(node_id)

        for target_id in sorted(graph[node_id]):
            if target_id not in indexes:
                visit(target_id)
                lowlinks[node_id] = min(lowlinks[node_id], lowlinks[target_id])
            elif target_id in stacked:
                lowlinks[node_id] = min(lowlinks[node_id], indexes[target_id])

        if lowlinks[node_id] == indexes[node_id]:
            component = []
            while True:
                current = stack.pop()
                stacked.remove(current)
                component.append(current)
                if current == node_id:
                    break
            if len(component) > 1 or component[0] in self_loops:
                components.append(set(component))

    for node_id in sorted(node_ids):
        if node_id not in indexes:
            visit(node_id)

    return components


def main():
    if len(sys.argv) != 3:
        print("usage: callgraph_json_to_svg.py INPUT OUTPUT", file=sys.stderr)
        return 2

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    with input_path.open() as input_file:
        data = json.load(input_file)

    nodes = sorted([node for node in data["nodes"] if is_crate_internal(node)], key=node_sort_key)
    groups = defaultdict(list)
    for node in nodes:
        groups[node["file"]].append(node)

    ordered_groups = sorted(groups.items(), key=group_sort_key)
    positions = {}
    group_headers = []
    max_group_size = 0
    for col, (crate, group_nodes) in enumerate(ordered_groups):
        x = MARGIN_X + col * (NODE_WIDTH + X_GAP)
        group_headers.append((crate, x, len(group_nodes)))
        max_group_size = max(max_group_size, len(group_nodes))
        for row, node in enumerate(group_nodes):
            y = MARGIN_Y + row * (NODE_HEIGHT + Y_GAP)
            positions[node["id"]] = (x, y)

    width = MARGIN_X * 2 + len(ordered_groups) * NODE_WIDTH + max(0, len(ordered_groups) - 1) * X_GAP
    height = MARGIN_Y + max_group_size * (NODE_HEIGHT + Y_GAP) + MARGIN_Y

    workspace_ids = {node["id"] for node in nodes}
    edges = sorted(
        [
            edge
            for edge in data["edges"]
            if edge["source"] in workspace_ids and edge["target"] in workspace_ids
        ],
        key=lambda edge: (edge["source"], edge["target"], edge.get("kind", "")),
    )
    # Separate every incoming and outgoing connection along the node boundary.
    incidents = defaultdict(list)
    for index, edge in enumerate(edges):
        for endpoint in ("source", "target"):
            incidents[edge[endpoint]].append((index, endpoint))
    ports = {}
    for connections in incidents.values():
        for rank, connection in enumerate(connections):
            ports[connection] = 6 + (NODE_HEIGHT - 12) * (rank + 1) / (len(connections) + 1)
    bottom = height - MARGIN_Y
    width += X_GAP
    long_edges = [index for index, edge in enumerate(edges)
                  if abs(positions[edge["source"]][0] - positions[edge["target"]][0])
                  > NODE_WIDTH + X_GAP]
    height += (max(long_edges, default=0) + 1) * 10
    cycle_components = find_cycle_components(workspace_ids, edges)
    cycle_node_ids = set().union(*cycle_components) if cycle_components else set()
    cycle_component_by_node = {
        node_id: index
        for index, component in enumerate(cycle_components)
        for node_id in component
    }
    for node in nodes:
        node["is_cycle"] = node["id"] in cycle_node_ids
    rendered_edges = [
        render_edge(
            edge["source"],
            edge["target"],
            positions,
            cycle_component_by_node.get(edge["source"]) == cycle_component_by_node.get(edge["target"])
            and edge["source"] in cycle_node_ids,
            index, ports, bottom,
        )
        for index, edge in enumerate(edges)
    ]

    rendered_nodes = [render_node(node, *positions[node["id"]]) for node in nodes]
    rendered_headers = [
        f'  <text x="{x}" y="82" fill="#0f172a" font-size="16" font-weight="700">{text(crate.removeprefix("shogi_legality_lite/"))}</text>\n'
        f'  <text x="{x}" y="102" fill="#64748b" font-size="11">{count_text(count)}</text>'
        for crate, x, count in group_headers
    ]

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <marker id="arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="userSpaceOnUse">
      <path d="M 0 0 L 8 4 L 0 8 z" fill="#64748b"/>
    </marker>
    <marker id="cycle-arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="userSpaceOnUse">
      <path d="M 0 0 L 8 4 L 0 8 z" fill="#dc2626"/>
    </marker>
  </defs>
  <style>
    text {{ font-family: sans-serif; }}
    .edge {{ cursor: crosshair; }}
    #edges:has(.edge:hover) .edge:not(:hover), #edges:has(.edge:focus) .edge:not(:focus) {{ opacity: .12; }}
    .edge:hover .wire, .edge:focus .wire {{ stroke-width: 3.5; }}
  </style>
  <rect width="100%" height="100%" fill="white"/>
  <text x="{MARGIN_X}" y="28" fill="#0f172a" font-size="20" font-weight="700">shogi_legality_lite legality callgraph</text>
  <text x="{MARGIN_X}" y="50" fill="#64748b" font-size="12">{len(nodes)} nodes, {len(edges)} internal edges, {cycle_count_text(len(cycle_components))}. Columns are source files; mate/perft are omitted.</text>
{chr(10).join(rendered_headers)}
  <g id="edges">
{chr(10).join(rendered_edges)}
  </g>
  <g id="nodes">
{chr(10).join(rendered_nodes)}
  </g>
</svg>
"""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(svg)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
