"""Fixed-denominator sizing; observations never become universal bounds."""
import json
from compact_evidence import allocation


def project(contract, indexes, audit_receipts, pack_receipts):
    p = contract['projection']
    adapter = indexes['adapter']
    entries = adapter['entries']
    world = [e for e in entries if e['path'].startswith('journals/') and
             e['path'].split('/')[1].startswith(('world_', 'stress_'))]
    assays = [e for e in entries if e['path'].startswith(('journals/assays_0/', 'journals/assays_1/'))]
    world_count = p['worlds'] * (2 * p['ticks'] + 4)
    assay_count = 2 * p['assays'] + 2 * p['assay_batches']
    total_records = world_count + assay_count
    w = max(e['compressed_bytes'] for e in world)
    a = max(e['compressed_bytes'] for e in assays)
    raw_projection = (world_count * max(e['raw_bytes'] for e in world) +
                      assay_count * max(e['raw_bytes'] for e in assays))
    block = p['allocation_block_bytes']
    payload = world_count * w + assay_count * a
    rounded = world_count * allocation(w, block) + assay_count * allocation(a, block)
    index = total_records * p['index_bytes_per_projected_record']
    filesystem = total_records * p['filesystem_metadata_bytes_per_projected_record']
    other = p['fixed_other_evidence_reserve_bytes']
    scratch = p['bounded_raw_scratch_bytes']
    emergency = p['retained_emergency_raw_bytes']
    peak = rounded + index + filesystem + other + scratch + emergency
    compression_rate = max(pack_receipts[n]['seconds'] / pack_receipts[n]['result']['source_raw_bytes']
                           for n in indexes)
    verification_rate = max(audit_receipts[n]['elapsed_seconds'] /
                            audit_receipts[n]['independent_byte_audit']['raw_bytes'] for n in indexes)
    overhead_seconds = raw_projection * (compression_rate + verification_rate) * p['timing_margin_multiplier']
    workflow_projection = p['campaign_native_seconds_projection'] + overhead_seconds
    return {'world_record_slots': world_count, 'assay_record_slots': assay_count,
            'total_record_slots': total_records, 'maximum_observed_world_gzip_bytes': w,
            'maximum_observed_assay_gzip_bytes': a, 'uncompressed_parent_projection_bytes': raw_projection,
            'compressed_record_projection_bytes': payload, 'rounded_record_projection_bytes': rounded,
            'index_reserve_bytes': index, 'filesystem_metadata_reserve_bytes': filesystem,
            'other_evidence_reserve_bytes': other, 'raw_scratch_reserve_bytes': scratch,
            'retained_emergency_raw_reserve_bytes': emergency,
            'observed_peak_storage_projection_bytes': peak, 'campaign_cap_bytes': p['campaign_output_cap_bytes'],
            'headroom_bytes': p['campaign_output_cap_bytes'] - peak,
            'observed_storage_envelope_pass': peak <= p['campaign_output_cap_bytes'],
            'compression_and_verification_projection_seconds_with_margin': overhead_seconds,
            'observed_total_workflow_projection_seconds': workflow_projection,
            'observed_workflow_projection_fits': workflow_projection <= p['workflow_cap_seconds'],
            'projection_is_certified_upper_bound': False, 'compression_guaranteed_for_unseen_records': False,
            'production_spool_integration_complete': False, 'whole_campaign_feasibility_pass': False,
            'new_native_rewrites': 0}
