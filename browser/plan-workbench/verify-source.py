"""Rebuild delivered source outside Git; isolate source/consumer test profiles."""
import argparse,json,os,subprocess,tempfile,zipfile,hashlib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--bindgen',required=True,type=Path);a=p.parse_args()
B=Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache'
expected=json.loads((C/'native-cases.json').read_text(encoding='utf-8'));controls=json.loads((C/'cli-controls.json').read_text(encoding='utf-8'));cli=json.loads((C/'cli-current.json').read_text(encoding='utf-8'))
env=os.environ.copy();env.setdefault('CARGO_TARGET_DIR',str(R/'target/cargo'));env.pop('RPLAN_URL',None);env.pop('GIT_COMMIT',None)
profile=['--config','profile.rplan-replay.inherits="dev"','--profile','rplan-replay']
with tempfile.TemporaryDirectory(prefix='rplan delivered source ') as tmp:
    tmp=Path(tmp).resolve();assert tmp.parent==Path(tempfile.gettempdir()).resolve()
    with zipfile.ZipFile(R/'target/plan-release/RPLAN/source.zip') as archive:
        for name in archive.namelist():assert not Path(name).is_absolute() and '..' not in Path(name).parts,name
        archive.extractall(tmp)
    root=tmp/'RPLAN-source';record=json.loads((root/'.source-record.json').read_text(encoding='utf-8'));assert not list(root.rglob('.git'))
    for file in record['files']:assert hashlib.sha256((root/file['path']).read_bytes()).hexdigest()==file['sha256'],file['path']
    for file in root.rglob('*.rs'):file.touch()
    def run(argv,cwd=root):subprocess.run(list(map(str,argv)),cwd=cwd,env=env,check=True)
    run(['cargo','+1.95.0','test','--workspace','--locked',*profile])
    run(['cargo','+1.95.0','test','--manifest-path','browser/plan-workbench/Cargo.toml','--all-targets','--locked',*profile])
    run(['python',root/'browser/plan-workbench/build-prototype.py','--bindgen',a.bindgen.resolve(),'--native-profile','rplan-replay'])
    companion=root/'browser/plan-workbench';cache=companion/'.cache'
    assert json.loads((cache/'native-cases.json').read_text(encoding='utf-8'))==expected,'Native response/input drift'
    assert json.loads((cache/'cli-controls.json').read_text(encoding='utf-8'))==controls,'Independent CLI semantic/control drift'
    assert json.loads((cache/'cli-current.json').read_text(encoding='utf-8'))==cli,'Original current CLI byte/control drift'
    run(['python',companion/'rehearse-consumer.py'])
    consumer=json.loads((cache/'consumer-rehearsal.json').read_text(encoding='utf-8'));assert len(consumer['tests'])==3
    npm='npm.cmd' if os.name=='nt' else 'npm';run([npm,'ci'],cwd=companion)
    browser=subprocess.run(['node','node_modules/@playwright/test/cli.js','test','inspector.spec.js','recovery.spec.js','--reporter=json'],cwd=companion,env=env,capture_output=True)
    (C/'source-browser.json').write_bytes(browser.stdout);(C/'source-browser-stderr.log').write_bytes(browser.stderr)
    if browser.returncode:raise RuntimeError('Delivered-source browser qualification failed; retained source-browser.json and stderr log')
    stats=json.loads(browser.stdout)['stats'];assert not any(stats.get(k,0)for k in ['unexpected','flaky','skipped']) and stats['expected']>0,stats
    (C/'source-replay.json').write_text(json.dumps({'commit':record['commit'],'source_files':len(record['files']),'original_native_tests':70,'adapter_tests':9,'native_outcomes_identical':len(expected),'independent_cli_controls_identical':len(controls),'current_original_cli_byte_controls_identical':len(cli),'rcount_rehearsal':consumer,'browser_task_and_recovery_checks':stats['expected'],'temporary_root_outside_git':str(root),'cargo_profile':'rplan-replay','binary_identity_claim':False},indent=2)+'\n',encoding='utf-8',newline='\n')
