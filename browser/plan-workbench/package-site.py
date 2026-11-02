"""Package exact complete tracked source and original notices from clean Git."""
import hashlib,html,json,shutil,subprocess,zipfile
from pathlib import Path
B=Path(__file__).resolve().parent;R=B.parents[1];P=R/'target/plan-prototype/RPLAN';S=R/'target/plan-release/RPLAN'
def run(*argv):return subprocess.check_output([str(x) for x in argv],cwd=R)
def sha(data):return hashlib.sha256(data).hexdigest()
def write(path,value):path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8',newline='\n')
def copy(source,dest):
    assert source.is_file() and not source.is_symlink(),source
    dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,dest)
assert not run('git','status','--porcelain').strip(),'Package only clean committed source'
run('python',B/'verify-foundation.py');head=run('git','rev-parse','HEAD').decode().strip();S.mkdir(parents=True,exist_ok=True)
runtime=['index.html','style.css','app.js','worker.js','example.json','disconnected.json','pkg/rplan_plan_workbench.js','pkg/rplan_plan_workbench_bg.wasm']
for name in runtime:
    if not name.startswith('pkg/'):assert run('git','show',head+':browser/plan-workbench/web/'+name)==(P/name).read_bytes(),('Stale/non-Git web source',name)
    copy(P/name,S/name)
source=[]
with zipfile.ZipFile(S/'source.zip','w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as archive:
    archive.comment=head.encode()
    for item in run('git','ls-tree','-rz','--full-tree',head).split(b'\0'):
        if not item:continue
        info,path=item.split(b'\t',1);mode,kind,oid=info.decode().split();path=path.decode()
        assert kind=='blob' and mode in ['100644','100755'],path
        data=run('git','cat-file','blob',oid);member=zipfile.ZipInfo('RPLAN-source/'+path,date_time=(1980,1,1,0,0,0));member.compress_type=zipfile.ZIP_DEFLATED;member.external_attr=int(mode,8)<<16;archive.writestr(member,data)
        source.append({'path':path,'mode':mode,'bytes':len(data),'sha256':sha(data)})
    archive.writestr('RPLAN-source/.source-record.json',json.dumps({'commit':head,'files':source},indent=2)+'\n')
write(S/'source-files.json',source)
for name,path in [('LICENSE','LICENSE'),('BUILD.md','browser/plan-workbench/BUILD.md'),('MODEL.md','browser/plan-workbench/MODEL.md'),('SOURCE-SCOPE.md','browser/plan-workbench/SOURCE-SCOPE.md')]:
    (S/name).write_bytes(run('git','show',head+':'+path))
metadata=json.loads(run('cargo','+1.95.0','metadata','--manifest-path','browser/plan-workbench/Cargo.toml','--locked','--format-version','1','--filter-platform','wasm32-unknown-unknown'))
packages=[]
for package in metadata['packages']:
    folder=Path(package['manifest_path']).parent;root=R if not package.get('source') else folder;notices=[];current=folder
    while True:
        notices.extend(p for p in current.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE','NOTICE','COPYRIGHT','UNLICENSE')))
        if current==root or root not in current.parents:break
        current=current.parent
    if package.get('license_file'):
        declared=folder/package['license_file'];assert declared.is_file(),declared;notices.append(declared)
    assert notices,('Missing original legal notice',package['name'])
    records=[]
    for index,notice in enumerate(sorted(set(notices))):
        dest=S/'licenses/dependencies'/(package['name']+'-'+package['version'])/(str(index)+'-'+notice.name);copy(notice,dest);records.append({'path':dest.relative_to(S).as_posix(),'sha256':sha(notice.read_bytes())})
    packages.append({'name':package['name'],'version':package['version'],'source':package.get('source') or 'RPLAN tracked source snapshot; original MIT LICENSE applies','license_metadata_for_software':package['license'],'notices':records})
write(S/'dependencies.json',packages)
rust=Path(run('rustc','+1.95.0','--print','sysroot').decode().strip())/'share/doc/rust';copy(rust/'COPYRIGHT-library.html',S/'rust/COPYRIGHT-library.html');rust_notices=[]
for path in sorted((rust/'licenses').rglob('*')):
    if path.is_file():
        dest=S/'rust/licenses'/path.relative_to(rust/'licenses');copy(path,dest);rust_notices.append({'path':dest.relative_to(S).as_posix(),'sha256':sha(path.read_bytes())})
assert rust_notices
write(S/'rust/provenance.json',{'rustc':run('rustc','+1.95.0','-vV').decode().strip(),'notices':rust_notices,'copyright_sha256':sha((S/'rust/COPYRIGHT-library.html').read_bytes())})
full_archive='https://github.com/giodl73-repo/RPLAN/archive/'+head+'.zip'
legal=['<h1>RPLAN plan workbench notices</h1><p>The complete tracked source snapshot retains the original RPLAN MIT LICENSE and existing public fixtures. Dependency/runtime original notices are separate. Pinned RCOUNT source is fetched only for compatibility rehearsal and is not mirrored into this artifact; no external dataset is added during packaging.</p><p><a href="LICENSE">Original RPLAN grant</a> · <a href="SOURCE-SCOPE.md">Source scope</a> · <a href="source-files.json">Exact tracked source files</a> · <a href="'+full_archive+'">Complete Git archive at this commit</a> · <a href="dependencies.json">Dependency record</a> · <a href="rust/COPYRIGHT-library.html">Rust runtime copyright</a> · <a href="rust/provenance.json">Rust notices</a></p>']
for package in packages:
    legal.append('<h2>'+html.escape(package['name']+' '+package['version'])+'</h2><p>'+html.escape(package['license_metadata_for_software'] or 'See original notices')+'</p>')
    for notice in package['notices']:legal.append('<p><a href="'+html.escape(notice['path'])+'">'+html.escape(notice['path'])+'</a></p>')
(S/'licenses.html').write_text('<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>RPLAN plan workbench licenses</title><body>'+''.join(legal)+'</body></html>',encoding='utf-8')
write(S/'build.json',{'schema':'rplan.plan-workbench.v1','commit':head,'repository':'https://github.com/giodl73-repo/RPLAN','full_git_archive':full_archive,'source_scope':'SOURCE-SCOPE.md','source_files':len(source),'source_zip_sha256':sha((S/'source.zip').read_bytes()),'wasm_sha256':sha((S/'pkg/rplan_plan_workbench_bg.wasm').read_bytes()),'rustc':run('rustc','+1.95.0','-vV').decode().strip(),'wasm_bindgen':'0.2.127','runtime_files':runtime,'actual_wasm_memory_max_bytes':67108864,'limits_and_semantics':'MODEL.md','dependency_groups':len(packages),'rust_notice_files':len(rust_notices),'scope':'Original package readers, hashes and audit/certificate semantics over supplied data. Caller profiles, timestamps, lineage and provenance are unverified. No legal approval, fairness, release approval, data authenticity or consumer adoption.'})
files=[{'path':p.relative_to(S).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(S.rglob('*')) if p.is_file() and p.name!='inventory.json']
runtime_bytes=sum((S/name).stat().st_size for name in runtime)
assert runtime_bytes<=1048576 and (S/'source.zip').stat().st_size<=2097152 and sum(x['bytes'] for x in files)<=8388608
write(S/'inventory.json',{'commit':head,'runtime_bytes':runtime_bytes,'files':files,'bytes_excluding_inventory':sum(x['bytes']for x in files)})
print(json.dumps({'site':str(S),'commit':head,'files':len(files)+1,'runtime_bytes':runtime_bytes,'source_files':len(source),'dependency_groups':len(packages),'rust_notices':len(rust_notices)}))
