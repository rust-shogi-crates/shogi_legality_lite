#!/usr/bin/env python3
import json
import sys
from collections import defaultdict
from pathlib import Path


EXCLUDED_SRC_FILES = {
    "shogi_legality_lite/src/mate_solver.rs",
    "shogi_legality_lite/src/perft.rs",
}


def is_legality_node(node):
    return node.get("is_workspace") and "/src/" in node["file"] and node["file"] not in EXCLUDED_SRC_FILES


def node_sort_key(node):
    return (node["file"], node.get("line") or 0, node["qualified_path"])


def find_cycle_components(nodes, edges):
    node_ids = {node["id"] for node in nodes}
    graph = {node_id: [] for node_id in node_ids}
    self_loops = set()
    for edge in edges:
        source = edge["source"]
        target = edge["target"]
        if source not in node_ids or target not in node_ids:
            continue
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
                components.append(sorted(component))

    for node_id in sorted(node_ids):
        if node_id not in indexes:
            visit(node_id)

    return components


def main():
    if len(sys.argv) != 3:
        print("usage: callgraph_cycles.py INPUT OUTPUT", file=sys.stderr)
        return 2

    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    with input_path.open() as input_file:
        data = json.load(input_file)

    nodes = sorted([node for node in data["nodes"] if is_legality_node(node)], key=node_sort_key)
    nodes_by_id = {node["id"]: node for node in nodes}
    edges = sorted(data["edges"], key=lambda edge: (edge["source"], edge["target"], edge.get("kind", "")))
    components = find_cycle_components(nodes, edges)

    result = {
        "scope": {
            "included": "workspace src nodes",
            "excluded_files": sorted(EXCLUDED_SRC_FILES),
        },
        "cycle_count": len(components),
        "cycles": [
            {
                "nodes": [
                    {
                        "id": node_id,
                        "qualified_path": nodes_by_id[node_id]["qualified_path"],
                        "file": nodes_by_id[node_id]["file"],
                        "line": nodes_by_id[node_id].get("line"),
                    }
                    for node_id in sorted(component, key=lambda node_id: node_sort_key(nodes_by_id[node_id]))
                ]
            }
            for component in sorted(
                components,
                key=lambda component: [node_sort_key(nodes_by_id[node_id]) for node_id in component],
            )
        ],
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w") as output_file:
        json.dump(result, output_file, indent=2)
        output_file.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
