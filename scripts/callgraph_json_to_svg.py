#!/usr/bin/env python3
import html
import json
import sys
from collections import defaultdict
from pathlib import Path


NODE_WIDTH = 300
NODE_HEIGHT = 44
X_GAP = 150
Y_GAP = 20
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
    return group[0]


def node_color(node):
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


def render_edge(source, target, positions):
    source_x, source_y = positions[source]
    target_x, target_y = positions[target]
    start_y = source_y + NODE_HEIGHT / 2
    end_y = target_y + NODE_HEIGHT / 2
    color = "#334155"
    opacity = "0.28"

    if source_x == target_x:
        start_x = source_x + NODE_WIDTH
        end_x = target_x + NODE_WIDTH
        loop_x = source_x + NODE_WIDTH + 48
        return (
            f'  <path d="M {start_x:.1f} {start_y:.1f} C {loop_x:.1f} {start_y:.1f}, '
            f'{loop_x:.1f} {end_y:.1f}, {end_x:.1f} {end_y:.1f}" '
            f'fill="none" stroke="{color}" stroke-width="1" opacity="{opacity}" marker-end="url(#arrow)"/>'
        )

    if source_x < target_x:
        start_x = source_x + NODE_WIDTH
        end_x = target_x
        bend = max(48, abs(end_x - start_x) * 0.45)
        first_x = start_x + bend
        second_x = end_x - bend
    else:
        start_x = source_x
        end_x = target_x + NODE_WIDTH
        bend = max(48, abs(start_x - end_x) * 0.45)
        first_x = start_x - bend
        second_x = end_x + bend

    return (
        f'  <path d="M {start_x:.1f} {start_y:.1f} C {first_x:.1f} {start_y:.1f}, '
        f'{second_x:.1f} {end_y:.1f}, {end_x:.1f} {end_y:.1f}" '
        f'fill="none" stroke="{color}" stroke-width="1" opacity="{opacity}" marker-end="url(#arrow)"/>'
    )


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
    rendered_edges = [
        render_edge(edge["source"], edge["target"], positions)
        for edge in edges
    ]

    rendered_nodes = [render_node(node, *positions[node["id"]]) for node in nodes]
    rendered_headers = [
        f'  <text x="{x}" y="82" fill="#0f172a" font-size="16" font-weight="700">{text(crate.removeprefix("shogi_legality_lite/"))}</text>\n'
        f'  <text x="{x}" y="102" fill="#64748b" font-size="11">{count_text(count)}</text>'
        for crate, x, count in group_headers
    ]

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <marker id="arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="strokeWidth">
      <path d="M 0 0 L 8 4 L 0 8 z" fill="#64748b"/>
    </marker>
  </defs>
  <rect width="100%" height="100%" fill="white"/>
  <text x="{MARGIN_X}" y="28" fill="#0f172a" font-size="20" font-weight="700">shogi_legality_lite legality callgraph</text>
  <text x="{MARGIN_X}" y="50" fill="#64748b" font-size="12">{len(nodes)} nodes, {len(edges)} internal edges. Columns are source files; mate/perft are omitted.</text>
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
