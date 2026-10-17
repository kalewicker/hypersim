"""Conservative class/root lifetime measurements; never grants science acceptance."""
import math

PROFILE='E2-rule-survival-measurement-v1'


def presence_spells(rows,field,identity):
    result=[];present=False
    for row in rows:
        now=any(c['rcf']==identity for c in row[field])
        if now and not present:result.append(dict(start=row['step'],end=None))
        if present and not now:result[-1]['end']=row['step']
        present=now
    return result


def duration(spell,horizon):
    if spell is None:return None
    end=spell['end'];start=spell['start']
    assert type(start) is int and 0<=start<=horizon and (end is None or type(end) is int and start<end<=horizon)
    length=(horizon if end is None else end)-start
    return dict(lower_bound=length,observed_death_age=None if end is None else length,right_censored=end is None)


def half_life_bounds(durations):
    """Bounds for the empirical first-spell median, with no censoring model.

    A censored spell can end at/after its observed lower bound, but its death
    time is unknown. Compute lower/upper order statistics; do not turn the
    horizon into a death or join recovered spells.
    """
    assert durations and all(d is None or isinstance(d,dict) for d in durations)
    n=len(durations)
    if any(d is None for d in durations):
        return dict(fixed_cohort=n,observed_births=sum(d is not None for d in durations),half_life=None,
            lower_bound=None,upper_bound=None,unmeasured=True,missing_births=True)
    for d in durations:
        lo=d['lower_bound'];death=d['observed_death_age']
        assert type(lo) is int and lo>=0 and type(d['right_censored']) is bool
        assert (death is None and d['right_censored']) or (type(death) is int and death==lo and not d['right_censored'])
    rank=(n+1)//2-1
    lower=sorted(d['lower_bound'] for d in durations)[rank]
    upper=sorted(math.inf if d['observed_death_age'] is None else d['observed_death_age'] for d in durations)[rank]
    exact=math.isfinite(upper) and lower==upper
    return dict(fixed_cohort=n,observed_births=n,half_life=lower if exact else None,lower_bound=lower,
        upper_bound=None if not math.isfinite(upper) else upper,unmeasured=not exact,missing_births=False)


def cell(receipt,identity,touch_steps):
    rows=receipt['timeline'];horizon=receipt['horizon']
    assert type(horizon) is int and horizon>0 and [r['step'] for r in rows]==list(range(horizon+1))
    assert rows[0]['state']['root_alive'] is True
    initial=receipt['initial_all_candidates'];assert all(identity!=c['rcf'] for c in initial),'initialized class excluded'
    candidates=[c for r in rows for c in r['genotype'] if c['rcf']==identity]
    if candidates:
        assert all(c['structural_rcf'] not in {v['structural_rcf'] for v in initial} for c in candidates),'priority-only novelty excluded'
    for r in rows:assert type(r['state']['root_alive']) is bool
    assert all(type(s) is int and 1<=s<=horizon for s in touch_steps) and touch_steps==sorted(touch_steps)
    genotype=presence_spells(rows,'genotype',identity);installed=presence_spells(rows,'installed',identity)
    root_death=next((r['step'] for r in rows if not r['state']['root_alive']),None)
    # Once a particular generational root dies, a later root cannot extend it.
    if root_death is not None:assert all(not r['state']['root_alive'] for r in rows[root_death:])
    fires=[r['step'] for r in rows[1:] for event in r['events'] if event['kind']=='world' and event['rcf']==identity]
    return dict(identity=identity,genotype_spells=genotype,installed_spells=installed,
        genotype_first_duration=duration(genotype[0] if genotype else None,horizon),
        installed_first_duration=duration(installed[0] if installed else None,horizon),
        root_duration=duration(dict(start=0,end=root_death),horizon),
        ordinary_fire_steps=fires,nontrivial=bool(fires),touch_steps=touch_steps,
        touch_intervals=[b-a for a,b in zip(touch_steps,touch_steps[1:])])


def cohort(cells,reference_exposure_interval=None):
    assert cells
    if reference_exposure_interval is not None:assert math.isfinite(reference_exposure_interval) and reference_exposure_interval>0
    genotype=half_life_bounds([c['genotype_first_duration'] for c in cells]);installed=half_life_bounds([c['installed_first_duration'] for c in cells])
    roots=half_life_bounds([c['root_duration'] for c in cells]);intervals=[d for c in cells for d in c['touch_intervals']]
    # Require an observed interval in every fixed cohort member; absence is
    # unmeasured rather than an infinite replacement interval.
    measured=all(c['touch_intervals'] for c in cells) and intervals and math.fsum(intervals)>0
    mean=math.fsum(intervals)/len(intervals) if measured else None
    eligible=all(c['nontrivial'] and c['genotype_first_duration'] is not None and c['installed_first_duration'] is not None for c in cells)
    point=eligible and genotype['half_life'] is not None and installed['half_life'] is not None
    literal=bool(point and mean is not None and genotype['half_life']>mean and installed['half_life']>mean)
    reference=bool(point and reference_exposure_interval is not None and genotype['half_life']>reference_exposure_interval and installed['half_life']>reference_exposure_interval)
    return dict(profile=PROFILE,fixed_cohort=len(cells),eligible_members=sum(c['nontrivial'] and c['genotype_first_duration'] is not None and c['installed_first_duration'] is not None for c in cells),
        genotype_first_half_life=genotype,installed_first_half_life=installed,root_half_life=roots,
        actual_touch_interval_count=len(intervals),mean_actual_region_touch_interarrival=mean,reference_exposure_interval=reference_exposure_interval,
        literal_lifetime_ratio_pass=literal,reference_exposure_lifetime_ratio_pass=reference,
        scientific_acceptance=False,sustained_claim=False,training_admitted=False)
