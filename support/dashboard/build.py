"""Render live catalogs and revision-attributed runner evidence as static HTML.

This module never interprets checker stdout as proof success. Only terminal
outcomes emitted by the shared runner's classifiers are accepted. Scope metadata
lives beside each target; coverage.json must account for every registered contract.
"""
import argparse
import base64
import collections
import copy
import csv
import datetime
import io
import json
import os
from pathlib import Path
import re

TARGETS = ['frontend', 'analyzer', 'emitter', 'simulator', 'synthesizer', 'cache']
HEADER = ['phase', 'contract', 'harness', 'mutation', 'status', 'elapsed_ms', 'exit_code', 'covers']


def same_revision(record, current):
    return (isinstance(record, dict) and record.get('schema') == 1
            and not record.get('dirty', True) and not record.get('upstreamDirty', True)
            and not current.get('dirty', True) and not current.get('upstreamDirty', True)
            and all(record.get(k) == current.get(k) and record.get(k)
                    for k in ('formal', 'upstream', 'locks')))


def load_results(contracts, current, bundles, run=None, attempt=None):
    """Keep failed, interrupted, and missing checks distinct; never mix CI attempts."""
    candidates = collections.defaultdict(list)
    runs = []
    for bundle in bundles:
        if bundle.get('schema') != 1:
            raise ValueError('Unsupported evidence schema')
        for raw in bundle.get('runs', []):
            identity = raw.get('identity') or {}
            ci = identity.get('ci') or {}
            if run and (str(ci.get('run')) != run or str(ci.get('attempt')) != attempt):
                continue
            attributable = same_revision(identity, current)
            summary = dict(line.split('=', 1) for line in raw.get('summary', '').splitlines() if '=' in line)
            # Metadata is corroborated with the runner summary, not the bundle name.
            if run and (summary.get('ci_run_id') != run or summary.get('ci_run_attempt') != attempt):
                attributable = False
            info = dict(identity=identity, status=summary.get('status', 'unknown'),
                        jobStatus=bundle.get('jobStatus') or 'unknown',
                        path=raw.get('path', ''), attributable=bool(attributable))
            runs.append(info)
            if not attributable:
                continue
            journal = raw.get('records', '')
            complete = journal[:journal.rfind('\n')+1]
            stream = csv.DictReader(io.StringIO(complete), delimiter='\t')
            if stream.fieldnames != HEADER:
                continue
            # Last entry per check wins within one append-only journal. A torn
            # final line cannot erase the preceding running entry or imply pass.
            latest = {}
            for record in stream:
                if None in record or any(record.get(k) is None for k in HEADER):
                    continue
                if record['phase'] not in ('positive', 'mutation') or record['status'] not in ('running', 'passed', 'failed', 'timeout'):
                    continue
                latest[(record['phase'], record['contract'], record['harness'], record['mutation'])] = record
            for c in contracts:
                if (c['target'], c['manifest'], c['adapter']) != (summary.get('target'), summary.get('manifest'), summary.get('adapter')):
                    continue
                for (phase, name, harness, mutation), r in latest.items():
                    if (name, harness) != (c['name'], c['harness']):
                        continue
                    if phase == 'positive' and (mutation or r['covers'] != str(len(c['covers']))):
                        continue
                    candidates[c['id']].append(dict(r, run=len(runs)-1))
    # A failure in any matching run is not hidden by an earlier/later pass.
    priority = {'failed': 4, 'timeout': 3, 'running': 2, 'passed': 1}
    for c in contracts:
        records = candidates[c['id']]
        positive = [r for r in records if r['phase'] == 'positive']
        c['execution'] = max(positive, key=lambda r: priority[r['status']])['status'] if positive else 'not-run'
        c['results'] = records
        c['mutationCounts'] = dict(collections.Counter(r['status'] for r in records if r['phase'] == 'mutation'))
    return runs


def source_location(c, files):
    """Resolve a literal or macro harness; explicitly fall back to its catalog."""
    components = c['harness'].split('::')
    name = components[-1]
    matches = []
    for path, text in files.items():
        for pattern in (r'\bfn\s+' + re.escape(name) + r'\b', r'\b' + re.escape(name) + r'\b'):
            m = re.search(pattern, text)
            if m:
                score = 2 * (len(components) > 1 and Path(path).stem == components[-2]) + bool(re.match(r'\bfn\s+', m[0]))
                matches.append((score, path, text[:m.start()].count('\n')+1))
                break
    if matches:
        score = max(x[0] for x in matches)
        winners = [x for x in matches if x[0] == score]
        if len(winners) == 1:
            _, path, line = winners[0]
            return path, line, 'harness'
    return c['catalog'], 1, 'catalog'


def families(root, contracts):
    output = []
    by_target = collections.defaultdict(list)
    for c in contracts:
        by_target[c['target']].append(c)
    for target in TARGETS:
        entries = by_target[target]
        if not entries:
            continue
        metadata = root / 'targets' / target / 'coverage.json'
        if metadata.exists():
            rows = json.loads(metadata.read_text())['families']
            assigned = set()
            for original in rows:
                r = copy.deepcopy(original)
                r['localId'] = r['id']
                r['id'] = target + '/' + r['id']
                r['target'] = target
                r['proofIds'], r['nativeIds'], r['cbmcIds'] = [], [], []
                membership = r.pop('contracts')
                for method, names in membership.items():
                    for name in names:
                        found = [c for c in entries if (c['method'], c['name']) == (method, name)]
                        if len(found) != 1:
                            raise ValueError(f'Stale/ambiguous coverage entry: {target}/{method}/{name}')
                        c = found[0]
                        if c['id'] in assigned:
                            raise ValueError('Duplicate scope assignment: ' + c['id'])
                        assigned.add(c['id'])
                        c['family'] = r['id']
                        r[{'kani':'proofIds','direct':'nativeIds','cbmc':'cbmcIds'}[method]].append(c['id'])
                r['testFunctions'] = []
                r['model'] = target == 'analyzer' and r['localId'] in ('bitwise','shift','equality','ordering','arithmetic','reduction','resize','selection','conditional','logical','nonlinear','foundation')
                output.append(r)
            missing = {c['id'] for c in entries} - assigned
            if missing:
                raise ValueError('Update targets/' + target + '/coverage.json for: ' + ', '.join(sorted(missing)))
        else:
            raise ValueError('Missing scope metadata: ' + str(metadata))
    return output


def scope_docs(root, documents):
    """Use maintained harness comments as the canonical assumptions and limits."""
    result = []
    for document in documents:
        path = Path(document['path'])
        if path.is_absolute() or '..' in path.parts or path.parts[0] not in ('targets', 'semantics'):
            raise ValueError('Invalid scope documentation path: ' + str(path))
        lines = []
        for line in (root / path).read_text().splitlines():
            if line.startswith('//!'):
                lines.append(line[3:].strip())
            elif not line.strip():
                if lines:
                    break
            else:
                break
        if not lines:
            raise ValueError('Missing scope documentation header: ' + str(path))
        for paragraph in re.split(r'\n\s*\n', '\n'.join(lines)):
            result.append(document['label'] + ': ' + ' '.join(paragraph.split()))
    return result


def build(root, output, evidence=None, run=None, attempt=None):
    contracts = json.loads((output / 'catalog.json').read_text())
    identity = json.loads((output / 'identity.json').read_text())
    ids = [c['id'] for c in contracts]
    if len(set(ids)) != len(ids):
        raise ValueError('Duplicate catalog IDs')
    source_files = {}
    for target in TARGETS:
        for directory, dirs, files in os.walk(root / 'targets' / target):
            dirs[:] = [d for d in dirs if d not in ('target', 'examples', '.git')]
            for name in files:
                if not name.endswith('.rs'):
                    continue
                p = Path(directory) / name
                rel = str(p.relative_to(root))
                if '/src/bin/' not in rel:
                    source_files[rel] = p.read_text()
    for c in contracts:
        domain = '/kani/' if c['method']=='kani' else '/tests/' if c['method']=='direct' else '/cbmc/'
        candidates = {p:t for p,t in source_files.items() if p.startswith('targets/'+c['target']+'/') and domain in p}
        c['source'], c['line'], c['sourceKind'] = source_location(c, candidates)
    rows = families(root, contracts)
    for row in rows:
        row['assumptions'].extend(scope_docs(root, row.get('scopeDocs', [])))
        for path in row['files']:
            if '/tests/' not in path or '/examples/' in path:
                continue
            text = (root / path).read_text()
            for m in re.finditer(r'#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+(\w+)', text):
                row['testFunctions'].append(dict(id=m[1], source=path, line=text[:m.start()].count('\n')+1))
    sources = {}
    for p in sorted({c['source'] for c in contracts} | {c['catalog'] for c in contracts} | {p for r in rows for p in r['files']}):
        text = (root / p).read_text()
        lines = text.splitlines()
        positions = {c['line'] for c in contracts if c['source'] == p}
        positions.update(t['line'] for r in rows for t in r['testFunctions'] if t['source'] == p)
        sources[p] = dict(head='\n'.join(lines[:26]), lines=len(lines), snippets={
            str(line): dict(start=max(1,line-5), text='\n'.join(lines[max(0,line-6):line+25]))
            for line in positions})
    bundles = []
    if evidence:
        evidence = Path(evidence)
        paths = [evidence] if evidence.is_file() else sorted(evidence.rglob('dashboard-evidence.json'))
        bundles = [json.loads(p.read_text()) for p in paths]
    runs = load_results(contracts, identity, bundles, run, attempt)
    pipeline = []
    for target in TARGETS:
        entries = [c for c in contracts if c['target']==target]
        pipeline.append(dict(id=target, name=target.capitalize(), stage=f'{len(pipeline)+1:02}',
                             counts=dict(collections.Counter(c['method'] for c in entries)),
                             kind=' / '.join(m.upper() if m!='direct' else 'Direct' for m in ('kani','cbmc','direct') if any(c['method']==m for c in entries)),
                             scope=' / '.join(f'{sum(c["method"]==m for c in entries)} {m.upper() if m!="direct" else "Direct"}' for m in ('kani','cbmc','direct') if any(c['method']==m for c in entries)),
                             path=entries[0]['catalog']))
    snapshot = dict(schema='1', title='Veryl Formal', generatedAt=datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    formal=identity['formal'], upstream=identity['upstream'], identity=identity,
                    counts=dict(kani=sum(c['method']=='kani' for c in contracts), direct=sum(c['method']=='direct' for c in contracts), cbmc=sum(c['method']=='cbmc' for c in contracts), families=len(rows)),
                    families=rows, contracts=contracts, sources=sources, pipeline=pipeline, runs=runs,
                    licenses={p.stem:p.read_text() for p in sorted((root / 'support/dashboard/assets').glob('*-OFL.txt'))})
    (output / 'snapshot.json').write_text(json.dumps(snapshot, ensure_ascii=False, indent=2)+'\n')
    template = (root / 'support/dashboard/template.html').read_text()
    template = template.replace('<!-- STYLES -->', (root / 'support/dashboard/style.css').read_text()).replace('<!-- APP -->', (root / 'support/dashboard/app.js').read_text())
    def embed_asset(match):
        path = root / 'support/dashboard/assets' / match[1]
        mime = 'image/png' if path.suffix == '.png' else 'font/ttf'
        return 'data:' + mime + ';base64,' + base64.b64encode(path.read_bytes()).decode('ascii')
    template = re.sub(r'\{\{ASSET:([A-Za-z0-9._-]+)\}\}', embed_asset, template)
    serialized = json.dumps(snapshot, ensure_ascii=False, separators=(',',':')).replace('<','\\u003c')
    (output / 'index.html').write_text(template.replace('<!-- SNAPSHOT -->', serialized))
    print(json.dumps(dict(counts=snapshot['counts'], attributedRuns=sum(r['attributable'] for r in runs), output=str(output)), ensure_ascii=False))
    return snapshot


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--evidence', type=Path)
    parser.add_argument('--run')
    parser.add_argument('--attempt')
    args = parser.parse_args()
    build(args.root, args.output, args.evidence, args.run, args.attempt)
