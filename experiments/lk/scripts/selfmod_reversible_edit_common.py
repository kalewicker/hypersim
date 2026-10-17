"""Independent exact material-addition verification for the named native reversion."""
import copy
from audit_selfmod_birth import require
from verify_selfmod_second_edit_interruption import native_order
from selfmod_third_edit_common import ThirdState


def verify_target(edited, faithful, target, contract):
    child = edited['selection']['child']
    before, after = faithful['initial'], edited['initial']
    order = native_order(before, child)
    require([n['variable'] for n in order] == [1], 'unique retained var1; var0 absent')
    require(contract['third_edit']['operator'] == 2 and contract['third_edit']['parameters'] == [0]*5, 'named native parameters')
    added = target['added_constraint']
    require(target == dict(operator=2, target_variable=0, target_label=0, child=child,
                          native_pnode_order=order, added_constraint=added,
                          original_parameters=[0]*5, effective_parameters=[0]*5,
                          scheduler_draws=0), 'exact named native target receipt')
    require(before.keys() == after.keys(), 'snapshot coverage')
    for key in before.keys() - {'rules', 'genes'}:
        require(before[key] == after[key], 'unchanged paid snapshot '+key)
    require(len(before['genes']) == len(after['genes']), 'same genealogy coverage')
    for f, e in zip(before['genes'], after['genes']):
        if f['id'] != child:
            require(f == e, 'all sibling and older genes unchanged')
        else:
            require(e['id'] == child and e['spec'] == contract['composite_spec'] and
                    e['class_hash'] == contract['composite_class_hash'], 'exact reverted composite')
            for key in f.keys() - {'spec', 'class_hash'}:
                require(f[key] == e[key], 'unchanged child metadata '+key)
    old, new = before['rules'], after['rules']
    require(old.keys() == new.keys() == {'nodes', 'edges', 'label_index', 'node_to_edges', 'step'}, 'rule-store coverage')
    require(old['step'] == new['step'], 'same rule clock')
    additions = {}
    for field in ['nodes', 'edges']:
        f, e = old[field], new[field]
        require(len(e) in [len(f), len(f)+1], 'at most one allocated '+field+' slot')
        changed = []
        for i, slot in enumerate(e):
            previous = f[i] if i < len(f) else None
            if previous != slot:
                require(i > 0 and slot['value'] is not None and
                        (previous is None or previous['value'] is None), 'only vacant/new slots filled')
                require(slot['version'] == (1 if previous is None else previous['version']+1), 'native allocated generation')
                changed.append((i, slot))
        require(len(changed) == 1, 'exactly one '+field+' addition')
        index, slot = changed[0]
        additions[field] = (slot['version'] << 32) | index
    node, edge = additions['nodes'], additions['edges']
    require(added == node, 'exact added constraint identity')
    require(new['nodes'][node & 0xffffffff]['value'] == dict(
        label=1003, energy=0.0, generation=0,
        properties=[[3, {'Int': 0}], [4, {'Int': 0}]]), 'only var0 label0 material added')
    node_ref = lambda id_: dict(idx=id_ & 0xffffffff, version=id_ >> 32)
    require(new['edges'][edge & 0xffffffff]['value'] == dict(
        label=2002, nodes=[node_ref(child), node_ref(node)]), 'only child-constraint role edge added')
    index = copy.deepcopy(old['label_index'])
    index.setdefault('2002', []).append(node_ref(edge))
    require(new['label_index'] == index, 'exact native label index addition')
    # SecondaryMap may extend to the newly allocated node index. All intermediate
    # vacant entries must remain empty; every other adjacency entry stays exact.
    adjacency = copy.deepcopy(old['node_to_edges'])
    while len(adjacency) <= (node & 0xffffffff):
        adjacency.append(dict(value=None, version=0))
    adjacency[node & 0xffffffff] = dict(value=[node_ref(edge)], version=node >> 32)
    adjacency[child & 0xffffffff]['value'].append(node_ref(edge))
    require(new['node_to_edges'] == adjacency, 'exact native adjacency additions')
    require(sorted(n['variable'] for n in native_order(after, child)) == [0, 1], 'both constraints retained')
    return dict(exact_material_edit_verified=True, added_constraint=node, added_role_edge=edge,
                target_variable=0, target_label=0, retained_label1=True)
