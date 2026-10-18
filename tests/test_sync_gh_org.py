import os,subprocess,pathlib,tempfile,shutil
SCRIPT = os.environ.get('SYNC_SCRIPT', str(pathlib.Path(__file__).resolve().parents[1] / 'scripts' / 'sync-gh-org.sh'))
B=pathlib.Path(tempfile.mkdtemp(prefix='sync-gh-org-test-'))
remote=B/'remote'; remote.mkdir(); root=B/'local'; root.mkdir(); home=B/'home'; home.mkdir(); bin=B/'bin'; bin.mkdir()
def run(*args,cwd=None,env=None,check=True):
 p=subprocess.run(args,cwd=cwd,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 if check and p.returncode: raise RuntimeError('failed '+str(args)+'\n'+p.stdout)
 return p
for name in ['repo1','repo2']:
 run('git','init','-q','--bare',str(remote/(name+'.git')))
 r=B/('seed-'+name);run('git','clone','-q',str(remote/(name+'.git')),str(r))
 run('git','-C',str(r),'config','user.name','Tester');run('git','-C',str(r),'config','user.email','test@example.com')
 (r/'hello.txt').write_text('base\n');run('git','-C',str(r),'add','-A');run('git','-C',str(r),'commit','-qm','base')
 run('git','-C',str(r),'branch','-M','main');run('git','-C',str(r),'push','-q','-u','origin','main')
 run('git','--git-dir',str(remote/(name+'.git')),'symbolic-ref','HEAD','refs/heads/main')
 run('git','-C',str(r),'tag','v1');run('git','-C',str(r),'push','-q','origin','v1')
# rewrite URLs via separate HOME global config
run('git','config','--file',str(home/'.gitconfig'),'url.file://'+str(remote)+'/.insteadOf','https://github.com/testorg/')
mock='''#!/usr/bin/env bash
case "$1" in
 api) printf 'repo1\\nrepo2\\n';;
 repo)
   if [[ "$2" == view ]]; then
     name=${3#testorg/}; test -e "$MOCK_REMOTE/$name.git" && printf '%s\\n' "$name"
   elif [[ "$2" == clone ]]; then
     git clone -q "https://github.com/$3.git" "$4"
   else exit 3; fi ;;
 *) exit 3;;
esac
'''
(bin/'gh').write_text(mock);(bin/'gh').chmod(0o755)
env=dict(os.environ,HOME=str(home),PATH=str(bin)+':'+os.environ['PATH'],MOCK_REMOTE=str(remote),GIT_CONFIG_NOSYSTEM='1')
run('git','clone','-q','https://github.com/testorg/repo1.git',str(root/'repo1'),env=env)
r=root/'repo1';run('git','-C',str(r),'config','user.name','Tester');run('git','-C',str(r),'config','user.email','test@example.com')
# stash original modified code
(r/'hello.txt').write_text('stashed\n');run('git','-C',str(r),'stash','push','-qm','original stash');stash=run('git','-C',str(r),'rev-parse','refs/stash').stdout.strip()
# leave dirty files
(r/'new.txt').write_text('valuable untracked\n')
# branch not checked out, committed ahead
run('git','-C',str(r),'branch','topic')
# detached worktree
wt=B/'extra-wt';run('git','-C',str(r),'worktree','add','--detach',str(wt),'HEAD');(wt/'detached.txt').write_text('valuable detached\n')
# remote edit ahead different file from local dirty
seed=B/'seed-repo1';(seed/'remote.txt').write_text('remote advance\n');run('git','-C',str(seed),'add','-A');run('git','-C',str(seed),'commit','-qm','remote advance');run('git','-C',str(seed),'push','-q','origin','main')
# dry run must not create .org-sync, commits, or repo2 clone
before=run('git','-C',str(r),'rev-parse','HEAD').stdout.strip()
p=run('bash',SCRIPT,'testorg','--root',str(root),'--dry-run',env=env)
assert 'would clone' in p.stdout and 'dirty=yes' in p.stdout
assert not (root/'.org-sync').exists() and not (root/'repo2').exists()
assert run('git','-C',str(r),'rev-parse','HEAD').stdout.strip()==before
print('DRY_RUN_OK')
# mutating run
p=run('bash',SCRIPT,'testorg','--root',str(root),'--all','--checkpoint','--backup-remote','--verify','--report',env=env,check=False)
print('SYNC_EXIT',p.returncode);print(p.stdout[-5500:]);assert p.returncode==0
assert (root/'repo2'/'hello.txt').exists()
assert (r/'new.txt').exists()
assert (wt/'detached.txt').exists()
assert run('git','-C',str(r),'rev-parse','refs/stash').stdout.strip()==stash
assert (r/'remote.txt').exists()
refs=run('git','--git-dir',str(remote/'repo1.git'),'for-each-ref','--format=%(refname)','refs/heads/sync','refs/tags/sync-remote').stdout
assert 'refs/heads/sync/local/' in refs and 'refs/heads/sync/stash/' in refs and 'refs/heads/sync/detached/' in refs,refs
# Both remote branches and the tag were untouched or incorporated in main.
assert 'refs/heads/sync/remote/' not in refs and 'refs/tags/sync-remote/' not in refs,refs
print('MUTATION_OK','reports',len(list((root/'.org-sync'/'reports').glob('*.md'))))
# conflicting merge: push remote newer commit for same file and local change
p2root=B/'clash';p2root.mkdir()
run('git','clone','-q','https://github.com/testorg/repo1.git',str(p2root/'repo1'),env=env)
clash=p2root/'repo1';run('git','-C',str(clash),'config','user.name','Tester');run('git','-C',str(clash),'config','user.email','test@example.com')
(clash/'hello.txt').write_text('local conflict\n')
# another clone remote
conf=B/'conflict-seed';run('git','clone','-q','https://github.com/testorg/repo1.git',str(conf),env=env)
run('git','-C',str(conf),'config','user.name','Tester');run('git','-C',str(conf),'config','user.email','test@example.com');(conf/'hello.txt').write_text('remote conflict\n')
run('git','-C',str(conf),'add','-A');run('git','-C',str(conf),'commit','-qm','remote conflict');run('git','-C',str(conf),'push','-q','origin','main',env=env)
p=run('bash',SCRIPT,'testorg','--root',str(p2root),'--repo','repo1','--checkpoint','--backup-remote','--verify','--report',env=env,check=False)
assert p.returncode!=0,p.stdout
assert 'merge conflict' in p.stdout.lower(),p.stdout
assert run('git','-C',str(clash),'ls-files','-u').stdout.strip()
assert 'local conflict' in (clash/'hello.txt').read_text() and 'remote conflict' in (clash/'hello.txt').read_text()
# Main never needs a duplicate backup: non-force merges keep prior main reachable.
print('CONFLICT_PRESERVED_OK')


# A genuinely diverged, unmerged remote branch is selected for backup.
seed2=B/'seed-repo2'
run('git','-C',str(seed2),'switch','-qc','unmerged-feature')
(seed2/'feature.txt').write_text('feature base\n')
run('git','-C',str(seed2),'add','-A')
run('git','-C',str(seed2),'commit','-qm','create feature')
run('git','-C',str(seed2),'push','-q','origin','unmerged-feature')
feature_root=B/'feature-sync';feature_root.mkdir()
run('git','clone','-q','https://github.com/testorg/repo2.git',str(feature_root/'repo2'),env=env)
feature_local=feature_root/'repo2'
run('git','-C',str(feature_local),'config','user.name','Tester')
run('git','-C',str(feature_local),'config','user.email','test@example.com')
run('git','-C',str(feature_local),'switch','-q','-c','unmerged-feature','origin/unmerged-feature')
(feature_local/'local-only.txt').write_text('feature-local\n')
run('git','-C',str(feature_local),'add','-A')
run('git','-C',str(feature_local),'commit','-qm','local feature')
(seed2/'remote-only.txt').write_text('feature-remote\n')
run('git','-C',str(seed2),'add','-A')
run('git','-C',str(seed2),'commit','-qm','remote feature')
run('git','-C',str(seed2),'push','-q','origin','unmerged-feature')
p=run('bash',SCRIPT,'testorg','--root',str(feature_root),'--repo','repo2','--checkpoint','--backup-remote','--verify','--report',env=env,check=False)
assert p.returncode==0,p.stdout
feature_refs=run('git','--git-dir',str(remote/'repo2.git'),'for-each-ref','--format=%(refname)','refs/heads/sync/remote').stdout
assert 'refs/heads/sync/remote/' in feature_refs,feature_refs
assert 'SKIPPED: remote backups:' in p.stdout
print('SELECTIVE_REMOTE_BACKUP_OK')

# Fail-closed safety gate: reject push of sync backup refs.
# Branch integration must not happen when the safety backup did not land.
gate_root=B/'backup-gate';gate_root.mkdir()
run('git','clone','-q','https://github.com/testorg/repo2.git',str(gate_root/'repo2'),env=env)
gated=gate_root/'repo2'
run('git','-C',str(gated),'config','user.name','Tester')
run('git','-C',str(gated),'config','user.email','test@example.com')
(gated/'gated-local.txt').write_text('must be preserved locally\n')
# Server-side pre-receive hook blocks only sync backup refs, not other pushes.
hook=remote/'repo2.git'/'hooks'/'pre-receive'
hook.write_text('#!/bin/sh\nwhile read old new ref; do\n case "$ref" in refs/heads/sync/*) exit 1;; esac\ndone\nexit 0\n')
hook.chmod(0o755)
previous=run('git','--git-dir',str(remote/'repo2.git'),'rev-parse','main').stdout.strip()
p=run('bash',SCRIPT,'testorg','--root',str(gate_root),'--repo','repo2','--checkpoint','--backup-remote','--verify',env=env,check=False)
assert p.returncode != 0 and 'local safety-ref push rejected' in p.stdout, p.stdout
assert run('git','--git-dir',str(remote/'repo2.git'),'rev-parse','main').stdout.strip()==previous
assert (gated/'gated-local.txt').exists()
assert run('git','-C',str(gated),'log','-1','--pretty=%s').stdout.strip().startswith('sync: checkpoint')
print('FAILED_BACKUP_BLOCKS_INTEGRATION_OK')
