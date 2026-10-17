#!/usr/bin/env python3
"""Instrument frozen reference functions without changing their physical checks."""
import ast
import copy
import hashlib


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def ast_hash(node):
    return hashlib.sha256(ast.dump(node, include_attributes=False).encode()).hexdigest()


def admission_function(module, source):
    tree = ast.parse(source)
    original = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'verify_cell')
    derived = copy.deepcopy(original)
    derived.args.args.append(ast.arg(arg='_checkpoint_hook'))
    derived.args.defaults.append(ast.Constant(None))
    hook = ast.parse('if _checkpoint_hook is not None: _checkpoint_hook(locals())').body[0]
    loops = [n for n in derived.body if isinstance(n, ast.For) and
             ast.unparse(n.target) == '(index, entry)' and ast.unparse(n.iter) == 'enumerate(entries)']
    require(len(loops) == 1, 'unique complete admission replay loop')
    index = derived.body.index(loops[0])
    derived.body.insert(index, copy.deepcopy(hook))
    boundaries = [n for n in ast.walk(loops[0]) if isinstance(n, ast.If) and ast.unparse(n.test) == 'phase == 45']
    require(len(boundaries) == 1 and ast.unparse(boundaries[0].body[-1]) == 'work = 0', 'unique completed tick boundary')
    boundaries[0].body.append(copy.deepcopy(hook))
    # Remove exactly the instrumentation and confirm every original statement is intact.
    restored = copy.deepcopy(derived)
    restored.args.args.pop(); restored.args.defaults.pop()
    restored.body.pop(index)
    boundary = next(n for n in ast.walk(restored) if isinstance(n, ast.If) and ast.unparse(n.test) == 'phase == 45')
    boundary.body.pop()
    require(ast_hash(restored) == ast_hash(original), 'original admission AST preserved')
    namespace = dict(module.__dict__)
    code = ast.fix_missing_locations(ast.Module(body=[derived], type_ignores=[]))
    exec(compile(code, '<frozen-admission-with-checkpoint-observer>', 'exec'), namespace)
    return namespace['verify_cell'], dict(original_ast_sha256=ast_hash(original),
        derived_ast_sha256=ast_hash(derived), restored_ast_sha256=ast_hash(restored),
        callback_sites=2, original_ast_restored_exactly=True)


def pilot_function(module, source, optimized_cell):
    tree = ast.parse(source)
    original = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'verify')
    derived = copy.deepcopy(original)
    loop = next(n for n in derived.body if isinstance(n, ast.For) and ast.unparse(n.iter) == 'cases()')
    indices = [i for i, n in enumerate(loop.body) if isinstance(n, ast.For) and
               ast.unparse(n.iter) == "seg['commits']"]
    require(len(indices) == 1, 'unique pilot prefix audit block')
    index = indices[0]
    removed = copy.deepcopy(loop.body[index:index+4])
    require(len(removed) == 4 and isinstance(removed[3], ast.Try) and
        ast.unparse(removed[1]).startswith('observer = o.LifetimeObserver') and
        ast.unparse(removed[2]) == 'c.apply = observer.apply' and
        ast.unparse(removed[3].body[-1]) == 'opportunities = observer.finish()', 'exact prefix/lifetime audit block')
    replacement = ast.parse('final, physics, opportunities, published = _optimized_cell(chain, modules, laws, cfg, policy, v, o, start)\ncount += published').body
    loop.body[index:index+4] = replacement
    restored = copy.deepcopy(derived)
    restored_loop = next(n for n in restored.body if isinstance(n, ast.For) and ast.unparse(n.iter) == 'cases()')
    restored_loop.body[index:index+2] = removed
    require(ast_hash(restored) == ast_hash(original), 'all original pilot checks preserved outside replay block')
    namespace = dict(module.__dict__, _optimized_cell=optimized_cell)
    code = ast.fix_missing_locations(ast.Module(body=[derived], type_ignores=[]))
    exec(compile(code, '<frozen-pilot-with-shared-replay>', 'exec'), namespace)
    return namespace['verify'], dict(original_ast_sha256=ast_hash(original),
        derived_ast_sha256=ast_hash(derived), restored_ast_sha256=ast_hash(restored),
        replaced_replay_blocks=1, original_ast_restored_exactly=True)


def optimized_cell(chain, modules, base, cfg, policy, v, o, start, derivations=None, handoff_config=None, descendant_config=None, transfer_environment=None, mutation_stress_policy=None):
    a, c, d, s = modules
    require(len(chain) == 1 or ((handoff_config is not None or descendant_config is not None or transfer_environment is not None) and len(chain) == 2), 'shared fresh segment or declared handoff recovery')
    commits = [x for seg in chain for x in seg['commits']]
    expected = list(range(0, 65, 8)) if len(chain)==1 else [0,8,16,24,24,32,40,48,56,64]
    require([x['marker']['cursor']['tick'] for x in commits] == expected, 'all nine shared checkpoints or declared recovery')
    evidence = v.flatten(chain, commits[-1])
    prefixes = {}
    for index, seg in enumerate(chain):
        for x in seg['commits']: prefixes.setdefault(x['marker']['cursor']['tick'], []).append((x,v.flatten(chain[:index+1],x)))
    checked = []

    def checkpoint(state):
        tick = len(state['rows'])
        if tick not in prefixes:
            return
        require(__import__('time').monotonic()-start <= 3600, 'pilot audit time cap')
        require(tick not in checked, 'duplicate shared tick')
        for commit, prefix in prefixes[tick]:
            require(c.graphs(commit['state']['frame']) == (state['nodes'], state['edges']), 'regenerated checkpoint graph')
            require(prefix['rows'] == state['rows'], 'regenerated checkpoint rows')
            require(prefix['lineage'] == {str(k): val for k, val in state['ancestry'].items()}, 'regenerated checkpoint ancestry')
            require(prefix['admission'] == dict(recipe=chain[0]['header']['recipe'], rounds=state['rounds'], records=state['requests']), 'regenerated checkpoint admission')
            v.audit_state(chain, commit, prefix)
            if policy is not None:
                require(commit['state']['native'].get('retained') == dict(policy=policy, samples=state['retained']), 'regenerated retained checkpoint samples')
            if descendant_config is not None:
                require(commit['state']['native'].get('descendant')==dict(config=state['descendant_config'],targets=state['descendant_targets']),'regenerated descendant target checkpoint state')
            if mutation_stress_policy is not None:
                require(commit['state']['native'].get('mutation_stress_policy')==mutation_stress_policy,'regenerated mutation stress policy')
            if transfer_environment is not None:
                require(commit['state']['native'].get('transfer_environment')==transfer_environment,'regenerated temporal configuration')
            if handoff_config is not None:
                require(commit['state']['native'].get('handoff_config')==state['handoff_config'],'regenerated handoff checkpoint config')
        checked.append(tick)

    hooked, receipt = admission_function(a, (chain[0]['directory'].parent/'admission_reference.py').read_text())
    if derivations is not None:
        derivations.append(receipt)
    original = a.verify_cell
    observer = o.LifetimeObserver(c, d, evidence, cfg)
    c.apply = observer.apply
    a.verify_cell = lambda *args, **kwargs: hooked(*args, **kwargs, _checkpoint_hook=checkpoint)
    try:
        final, physics = v.physical_audit(chain, commits[-1], modules, base, funded_config=cfg, retained_policy=policy, **(dict(handoff_config=handoff_config) if handoff_config is not None else {}), **(dict(descendant_config=descendant_config) if descendant_config is not None else {}), **(dict(transfer_environment=transfer_environment) if transfer_environment is not None else {}), **(dict(mutation_stress_policy=mutation_stress_policy) if mutation_stress_policy is not None else {}))
        opportunity = observer.finish()
    finally:
        a.verify_cell = original
        c.apply = observer.orig_apply
    require(checked == list(range(0, 65, 8)), 'complete regenerated checkpoint denominator')
    return final, physics, opportunity, len(commits)
