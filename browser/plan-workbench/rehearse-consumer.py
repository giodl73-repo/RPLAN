"""Run the compatibility policy's three tests against this RPLAN source."""
import argparse,datetime,hashlib,json,os,subprocess,tempfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--consumer',type=Path);a=p.parse_args()
B=Path(__file__).resolve().parent;R=B.parents[1];C=B/'.cache';C.mkdir(exist_ok=True)
PIN='3849b249b6d4f3016d387faef936d82899527141';URL='https://github.com/giodl73-repo/RCOUNT.git'
TESTS=['aggregates_summary_basic_into_rplan_districts','aggregation_transcript_records_matching_rctx_reference','aggregation_rejects_declared_crosswalk_hash_drift']
env=os.environ.copy();env.pop('GH_TOKEN',None);env['GODEBUG']='http2client=0';env.setdefault('CARGO_TARGET_DIR',str(R/'target/cargo'))
def run(argv,cwd):return subprocess.run(list(map(str,argv)),cwd=cwd,env=env,capture_output=True,check=True)
with tempfile.TemporaryDirectory(prefix='rplan consumer rehearsal ') as temporary:
    temporary=Path(temporary).resolve();assert temporary.parent==Path(tempfile.gettempdir()).resolve()
    consumer=a.consumer.resolve() if a.consumer else temporary/'RCOUNT'
    if not a.consumer:
        consumer.mkdir();run(['git','init','--quiet'],consumer);run(['git','remote','add','origin',URL],consumer);run(['git','fetch','--quiet','--depth','1','origin',PIN],consumer);run(['git','checkout','--quiet','--detach','FETCH_HEAD'],consumer)
    sha=run(['git','rev-parse','HEAD'],consumer).stdout.decode().strip();assert sha==PIN,sha
    # CLI overrides remain outside the consumer and never become a production pin.
    config=temporary/'rehearsal.toml'
    config.write_text('[patch."https://github.com/giodl73-repo/RPLAN.git"]\n'+''.join(name+' = { path = '+json.dumps((R/'crates'/name).as_posix())+' }\n'for name in ['rplan-core','rplan-io']),encoding='utf-8',newline='\n')
    flags=['--config',str(config),'--config','profile.rplan-consumer.inherits="dev"']
    # A new consumer root must not reuse first-party compile-time paths from a
    # prior temporary extraction; the normal dev-profile tests stay untouched.
    for file in (consumer/'crates').rglob('*.rs'):file.touch()
    tests=[]
    for name in TESTS:
        result=run(['cargo','+1.95.0','test','-p','rcount-district',name,'--profile','rplan-consumer',*flags],consumer)
        output=result.stdout.decode('utf-8');assert '1 passed; 0 failed' in output and 'test tests::'+name+' ... ok' in output,output
        (C/('consumer-'+name+'.log')).write_bytes(result.stdout+result.stderr);tests.append({'name':name,'actual_tests_passed':1})
    metadata=json.loads(run(['cargo','+1.95.0','metadata','--locked','--format-version','1',*flags],consumer).stdout)
    packages=[]
    for name in ['rplan-core','rplan-io']:
        package=next(pkg for pkg in metadata['packages'] if pkg['name']==name)
        assert package['source'] is None and Path(package['manifest_path']).resolve()==R/'crates'/name/'Cargo.toml'
        packages.append({'name':name,'version':package['version'],'source':'current supplied RPLAN source root','manifest_git_text_sha256':hashlib.sha256((R/'crates'/name/'Cargo.toml').read_bytes().replace(b'\r\n',b'\n')).hexdigest()})
    proof={'checked_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'consumer_repository':URL,'consumer_commit':sha,'tests':tests,'dependency_resolution':packages,'cargo_profile':'rplan-consumer','normal_profile_not_overwritten':True,'scope':'Local compatibility only; no production dependency adoption or BISECT change.'}
    (C/'consumer-rehearsal.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(proof))
