"""Capture deterministic original CLI output before/after portable helper extraction."""
import argparse,hashlib,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--binary',required=True,type=Path);p.add_argument('--output',required=True,type=Path);args=p.parse_args()
R=Path(__file__).resolve().parents[2];B=R/'browser/plan-workbench';F=B/'.cache/cli-fixtures';F.mkdir(parents=True,exist_ok=True)
fixture='crates/rplan-audit/fixtures/';profile='crates/rplan-audit/profiles/';cases=[['--help']]
for plan in ['grid3x3-valid.rplan','grid3x3-disconnected.rplan']:
    for constraints in ['plan-shape','plan-shape,population,contiguity','plan-shape,population,contiguity,splits,vra,geometry']:
        for context in [[],['--context',fixture+'grid3x3.rctx']]:
            cases.append(['audit','--plan',fixture+plan,*context,'--constraints',constraints,'--fixed-generated-at','2026-05-10T00:00:00Z'])
base=json.loads((R/(fixture+'grid3x3-valid.rplan')).read_text(encoding='utf-8'));policy=json.loads((R/(profile+'us-congressional-project-v1.json')).read_text(encoding='utf-8'))
for field,value in [('year',2021),('jurisdiction','XX'),('chamber','state-house')]:
    changed=dict(policy);changed[field]=value;path=F/(field+'.json');path.write_text(json.dumps(changed)+'\n',encoding='utf-8')
    cases.append(['audit','--plan',fixture+'grid3x3-valid.rplan','--legal-profile',path.relative_to(R).as_posix(),'--fixed-generated-at','2026-05-10T00:00:00Z'])
base['metadata']['chamber']='unknown';path=F/'unknown-chamber.rplan';path.write_text(json.dumps(base)+'\n',encoding='utf-8');cases.append(['audit','--plan',path.relative_to(R).as_posix(),'--fixed-generated-at','2026-05-10T00:00:00Z'])
cases.append(['audit','--plan',fixture+'grid3x3-valid.rplan','--legal-profile',profile+'incomplete-state-house-profile.json','--fixed-generated-at','2026-05-10T00:00:00Z'])
for certificate in ['grid3x3-valid-certificate.json','grid3x3-disconnected-certificate.json','grid3x3-missing-contiguity-certificate.json']:
    for plan in [[],['--plan',fixture+'grid3x3-valid.rplan'],['--plan',fixture+'grid3x3-disconnected.rplan']]:
        cases.append(['verify-certificate','--certificate',fixture+certificate,'--context',fixture+'grid3x3.rctx',*plan])
records=[]
for command in cases:
    def run():return subprocess.run([str(args.binary.resolve()),*command],cwd=R,capture_output=True)
    first=run();second=run();assert (first.returncode,first.stdout,first.stderr)==(second.returncode,second.stdout,second.stderr),command
    records.append({'arguments':command,'exit_code':first.returncode,'stdout':first.stdout.decode('utf-8'),'stderr':first.stderr.decode('utf-8')})
args.output.parent.mkdir(parents=True,exist_ok=True);data=(json.dumps(records,indent=2)+'\n').encode();args.output.write_bytes(data)
print(json.dumps({'controls':len(records),'each_repeated':2,'sha256':hashlib.sha256(data).hexdigest()}))
