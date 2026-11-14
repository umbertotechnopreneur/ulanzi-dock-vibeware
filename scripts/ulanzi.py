#!/usr/bin/env python3
# Project: UlanziDeck
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# Retain the copyright and permission notice. See LICENSE.
"""One console for setup, source builds, configuration and developer tools."""
import argparse
import hashlib
import os
from pathlib import Path
import platform
import re
import shutil
import shlex
import tempfile
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / 'scripts'
CACHE = Path(os.environ.get('XDG_CACHE_HOME', Path.home() / '.cache')) / 'ulanzideck'
COMMANDS = {
    'preflight': 'Check prerequisites and show what is missing',
    'setup': 'Stage locked Linux dependencies for a first-import scan',
    'build': 'Build programs from current source',
    'test-actions': 'Check the Linux reference actions with simulated button events',
    'configure': 'Open application settings without connecting to the dock',
    'run': 'Start the controller with the connected D200H',
    'msix': 'Build a Windows installer',
    'package': 'Package an existing build',
    'deb': 'Make a Debian package from a native Linux build',
    'icons': 'Prepare application icons',
    'msix-icons': 'Prepare Windows installer icons',
    'atlases': 'Make preview sheets from exported buttons',
    'obs-guide': 'Prepare the OBS preview and guide',
    'notices': 'Collect dependency licenses',
}


def run(args, *, capture=False, env=None):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, env=env,
                            text=True, capture_output=capture, check=True)
    return result.stdout.strip() if capture else ''


def require(name):
    path = shutil.which(name)
    if not path:
        raise RuntimeError(f'{name} is missing. See docs/DEVELOPMENT.md for prerequisites.')
    return path


def build_env():
    env = os.environ.copy()
    env['RUSTUP_AUTO_INSTALL'] = '0'
    env.setdefault('CARGO_BUILD_JOBS', str(min(4, os.cpu_count() or 1)))
    env.setdefault('CARGO_TARGET_DIR', str(CACHE / 'target'))
    env.setdefault('TMPDIR', str(CACHE / 'tmp'))
    for key in ('CARGO_TARGET_DIR', 'TMPDIR'):
        Path(env[key]).mkdir(parents=True, exist_ok=True)
    return env


def rust_ready():
    if not shutil.which('rustc'):
        return False
    try:
        version = run(['rustc', '--version'], capture=True,
                      env={**os.environ, 'RUSTUP_AUTO_INSTALL': '0'})
    except subprocess.CalledProcessError:
        return False
    match = re.search(r'rustc (\d+)\.(\d+)', version)
    return bool(match and tuple(map(int, match.groups())) >= (1, 90))


def preflight():
    checks = {name: bool(shutil.which(name)) for name in ('cargo', 'rustc')}
    checks['Rust 1.90 or newer'] = rust_ready()
    if platform.system() == 'Linux':
        checks.update({name: bool(shutil.which(name)) for name in ('gcc', 'pkg-config')})
        checks['xkbcommon development files'] = bool(shutil.which('pkg-config')) and subprocess.run(
            ['pkg-config', '--exists', 'xkbcommon'], cwd=ROOT).returncode == 0
    elif platform.system() == 'Windows':
        checks['PowerShell 7'] = bool(shutil.which('pwsh'))
        checks['MSVC developer shell'] = bool(shutil.which('cl'))
    else:
        checks['Xcode command line tools'] = bool(shutil.which('clang'))
    for name, ready in checks.items():
        print(f"  {'READY' if ready else 'MISSING'}  {name}")
    print('This checks the installed toolchain and build tools.')
    return all(checks.values())


def setup(options):
    if platform.system() != 'Linux':
        raise RuntimeError('This command stages Linux dependencies. See docs/DEVELOPMENT.md for other platforms.')
    parser = argparse.ArgumentParser(prog='ulanzi.py setup')
    parser.add_argument('stage', type=Path, help='Empty staging directory outside the repository')
    stage = parser.parse_args(options).stage.resolve()
    if stage.is_relative_to(ROOT):
        raise RuntimeError('Choose a staging directory outside the repository.')
    stage.mkdir(parents=True, exist_ok=True)
    if any(stage.iterdir()):
        raise RuntimeError('Choose an empty directory for a fresh import.')
    targets = {'x86_64': 'x86_64-unknown-linux-gnu', 'aarch64': 'aarch64-unknown-linux-gnu'}
    target = targets.get(platform.machine().lower())
    if target is None:
        raise RuntimeError(f'Unsupported setup architecture: {platform.machine()}')
    env = {**os.environ, 'CARGO_HOME': str(stage), 'RUSTUP_AUTO_INSTALL': '0'}
    run([require('cargo'), 'fetch', '--locked', '--target', target], env=env)
    print(f'Dependencies staged in {stage}.')
    print('Scan the new archives and extracted files with Windows Defender before using this directory.')


def package_deb(options):
    """Package a native Linux build; never install it or start the controller."""
    import struct
    import tomllib

    parser = argparse.ArgumentParser(prog='ulanzi.py deb', description=package_deb.__doc__)
    parser.add_argument('--binary', type=Path, help='Native release executable built from the current source')
    parser.add_argument('--output', type=Path, default=CACHE / 'dist')
    args = parser.parse_args(options)
    if platform.system() != 'Linux':
        raise RuntimeError('Debian packaging needs a native Debian/Ubuntu build host.')
    for tool in ('dpkg', 'dpkg-deb', 'dpkg-shlibdeps'):
        require(tool)
    arch = run(['dpkg', '--print-architecture'], capture=True)
    targets = {'amd64': ('x86_64-unknown-linux-gnu', 62), 'arm64': ('aarch64-unknown-linux-gnu', 183)}
    if arch not in targets:
        raise RuntimeError(f'Unsupported Debian architecture: {arch}')
    target, machine = targets[arch]
    env = build_env()
    binary = (args.binary or Path(env['CARGO_TARGET_DIR']) / 'release/ulanzi-dock-vibeware').resolve()
    with binary.open('rb') as stream:
        header = stream.read(20)
    if len(header) != 20 or header[:6] != b'\x7fELF\x02\x01' or struct.unpack('<H', header[18:20])[0] != machine:
        raise RuntimeError('The executable must be a native 64-bit Linux ELF for this package architecture.')
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f'ulanzi-dock-vibeware_{version}_{arch}.deb'
    if destination.exists() or destination.with_suffix('.deb.sha256').exists():
        raise RuntimeError(f'Refusing to overwrite an existing package: {destination}')
    with tempfile.TemporaryDirectory(prefix='deb-package-', dir=env['TMPDIR']) as temporary:
        staging = Path(temporary)
        tree = staging / 'package'

        def write(name, contents, executable=False):
            path = tree / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding='utf-8')
            path.chmod(0o755 if executable else 0o644)

        installed = tree / 'usr/lib/ulanzideck/ulanzi-dock-vibeware'
        installed.parent.mkdir(parents=True)
        shutil.copyfile(binary, installed)
        installed.chmod(0o755)
        write('usr/bin/ulanzideck', '''#!/bin/sh
# Copyright (c) 2026 Umberto Giacobbi. MIT License.
set -eu
binary=/usr/lib/ulanzideck/ulanzi-dock-vibeware
if [ "$#" -eq 0 ]; then set -- run; fi
# Desktop launches have no terminal. Preserve errors and panic backtraces locally.
case "$1" in
    run|configure|about|quit|launch)
        logdir="${XDG_STATE_HOME:-$HOME/.local/state}/ulanzideck/logs"
        if mkdir -p "$logdir"; then
            if log=$(umask 077; mktemp "$logdir/$1-XXXXXXXX.log"); then
                exec 2>>"$log"
                printf 'UlanziDeck launch: %s UTC | command=%s | pid=%s\n' "$(date -u +%FT%TZ)" "$1" "$$" >&2
            fi
        fi
        RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
        export RUST_BACKTRACE
        ;;
esac
if [ "$1" = launch ]; then
    if [ "$#" -ne 1 ]; then
        printf '%s\n' 'launch uses the installed default profile; use configure --config for a separate editor.' >&2
        exit 2
    fi
    set -- configure --start-controller
fi
case "$1" in
    run|configure|about|quit|brightness)
        profile="${XDG_CONFIG_HOME:-$HOME/.config}/ulanzideck/settings.json"
        # An explicit --config belongs to the caller; do not append a second one.
        for argument in "$@"; do
            case "$argument" in
                --config|--config=*) exec "$binary" "$@" ;;
            esac
        done
        mkdir -p "$(dirname "$profile")"
        exec "$binary" "$@" --config "$profile"
        ;;
    *) exec "$binary" "$@" ;;
esac
''', executable=True)
        write('usr/share/applications/ulanzideck.desktop', '''[Desktop Entry]
Type=Application
Name=UlanziDeck
Comment=Start my Ulanzi D200H controller and open settings
Exec=ulanzideck launch
Icon=ulanzideck
StartupWMClass=ulanzideck
Terminal=false
Categories=Utility;Settings;
''')
        write('usr/lib/systemd/user/ulanzideck-controller.service', '''# Copyright (c) 2026 Umberto Giacobbi. MIT License.
[Unit]
Description=UlanziDeck D200H controller
PartOf=graphical-session.target
After=graphical-session-pre.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
Type=simple
ExecStart=/usr/bin/ulanzideck run --non-interactive
Restart=on-failure
RestartSec=3
KillSignal=SIGINT
TimeoutStopSec=15

[Install]
WantedBy=graphical-session.target
''')
        icon = tree / 'usr/share/icons/hicolor/256x256/apps/ulanzideck.png'
        icon.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / 'assets/icons/ulanzi-dock-256.png', icon)
        write('usr/lib/udev/rules.d/70-ulanzideck.rules', '''# Copyright (c) 2026 Umberto Giacobbi. MIT License.
# Grant the active local desktop session access to this D200H only.
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2207", ATTRS{idProduct}=="0019", TAG+="uaccess"
''')
        write('DEBIAN/postinst', '''#!/bin/sh
set -e
if [ "$1" = configure ] && command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
fi
# Reconnect the dock after installation. Never start the controller as root.
''', executable=True)
        documentation = tree / 'usr/share/doc/ulanzi-dock-vibeware'
        documentation.mkdir(parents=True)
        for source, name in [('LICENSE', 'copyright'), ('README.md', 'README.md'),
                             ('docs/THIRD-PARTY.md', 'THIRD-PARTY.md'),
                             ('docs/ARTWORK.md', 'ARTWORK.md'), ('docs/PLATFORMS.md', 'PLATFORMS.md')]:
            shutil.copyfile(ROOT / source, documentation / name)
        shutil.copytree(ROOT / 'docs/licenses', documentation / 'licenses')
        shutil.copyfile(ROOT / 'assets/fonts/bitstream-vera-license.txt', documentation / 'bitstream-vera-license.txt')
        run([sys.executable, SCRIPTS / 'tools.py', 'notices', '--target', target,
             '--output', documentation / 'third-party-notices'], env=env)
        # dpkg-shlibdeps reads a source control stanza even with -O.
        debian = staging / 'debian'
        debian.mkdir()
        (debian / 'control').write_text('Source: ulanzi-dock-vibeware\nSection: utils\nPriority: optional\nMaintainer: Umberto Giacobbi <umberto.giacobbi@gmail.com>\n\nPackage: ulanzi-dock-vibeware\nArchitecture: any\nDescription: UlanziDeck controller\n', encoding='utf-8')
        result = subprocess.run(['dpkg-shlibdeps', '-O', '-e' + str(installed)],
                                cwd=staging, text=True, capture_output=True, check=True)
        dependencies = next(line.removeprefix('shlibs:Depends=') for line in result.stdout.splitlines()
                            if line.startswith('shlibs:Depends='))
        # These desktop backends are loaded dynamically and are invisible to ELF dependency scanning.
        dependencies += ', systemd, wireplumber, libglib2.0-bin, gnome-control-center, gnome-characters, xdg-desktop-portal-gnome, udev, libxkbcommon0, libxkbcommon-x11-0, libx11-6, libxcursor1, libxi6, libxrandr2, libwayland-client0, libwayland-cursor0'
        size = (sum(path.stat().st_size for path in (tree / 'usr').rglob('*') if path.is_file()) + 1023) // 1024
        write('DEBIAN/control', f'''Package: ulanzi-dock-vibeware
Version: {version}
Architecture: {arch}
Maintainer: Umberto Giacobbi <umberto.giacobbi@gmail.com>
Section: utils
Priority: optional
Homepage: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Installed-Size: {size}
Depends: {dependencies}
Description: UlanziDeck controller and configurator for the Ulanzi D200H
 Ten pages of shortcuts and thirteen themes. Independent MIT project
 by Umberto Giacobbi. Other dock models are unverified.
''')
        for directory in tree.rglob('*'):
            if directory.is_dir():
                directory.chmod(0o755)
            elif directory.is_file():
                executable = directory == installed or directory == tree / 'usr/bin/ulanzideck' or directory == tree / 'DEBIAN/postinst'
                directory.chmod(0o755 if executable else 0o644)
        pending = staging / destination.name
        run(['dpkg-deb', '--root-owner-group', '--build', tree, pending])
        # Publish a complete file atomically; never replace another run's package.
        with tempfile.NamedTemporaryFile(dir=output, prefix='.deb-', delete=False) as stream:
            publication = Path(stream.name)
            with pending.open('rb') as source:
                shutil.copyfileobj(source, stream)
            stream.flush()
            os.fsync(stream.fileno())
        try:
            publication.chmod(0o644)
            os.link(publication, destination)
        finally:
            publication.unlink()
        with destination.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        with destination.with_suffix('.deb.sha256').open('x', encoding='ascii') as stream:
            stream.write(f'{digest}  {destination.name}\n')
    print(f'Package ready: {destination}')


def offline_build_env():
    home = os.environ.get('CARGO_HOME')
    if not home or not Path(home).is_absolute() or not Path(home).is_dir():
        raise RuntimeError('Set CARGO_HOME to the prepared Cargo cache. See docs/DEVELOPMENT.md.')
    return build_env()


def dispatch(command, options):
    if command in ('preflight', 'build', 'configure') and options:
        raise RuntimeError(f'{command} does not accept extra options.')
    if command == 'preflight':
        if not preflight():
            raise RuntimeError('Prerequisites are missing. See docs/DEVELOPMENT.md.')
    elif command == 'setup':
        setup(options)
    elif command == 'build':
        if platform.system() == 'Windows':
            run([require('pwsh'), '-NoProfile', '-File', SCRIPTS / 'ulanzi.ps1', '-Command', 'build'])
        else:
            if not preflight():
                raise RuntimeError('Prerequisites are missing. See docs/DEVELOPMENT.md.')
            env = offline_build_env()
            run([require('cargo'), 'build', '--frozen', '--release', '--bins'], env=env)
    elif command == 'test-actions':
        if platform.system() != 'Linux':
            raise RuntimeError('The reference action suite targets Linux.')
        parser = argparse.ArgumentParser(prog='ulanzi.py test-actions')
        parser.add_argument('--config', type=Path, help='Read a local profile and compare it to the reference layout')
        args = parser.parse_args(options)
        env = offline_build_env()
        if args.config:
            env['ULANZI_OFFLINE_PROFILE'] = str(args.config.resolve(strict=True))
        else:
            env.pop('ULANZI_OFFLINE_PROFILE', None)
        run([require('cargo'), 'test', '--release', '--frozen', '--bin',
             'ulanzi-dock-vibeware', 'offline_actions', '--', '--nocapture'], env=env)
    elif command == 'deb':
        package_deb(options)
    elif command in ('configure', 'run'):
        if command == 'configure' and options:
            raise RuntimeError('configure does not accept extra options.')
        env = build_env()
        binary = (ROOT / 'artifacts/ulanzi-dock-vibeware.exe' if platform.system() == 'Windows'
                  else Path(env['CARGO_TARGET_DIR']) / 'release/ulanzi-dock-vibeware')
        if not binary.is_file():
            raise RuntimeError('Application is missing. Run build first.')
        run([binary, command, *options], env=env)
    elif command in ('msix', 'msix-icons', 'package'):
        run([require('pwsh'), '-NoProfile', '-File', SCRIPTS / 'ulanzi.ps1', '-Command', command, *options])
    else:
        run([sys.executable, SCRIPTS / 'tools.py', command, *options])


def main():
    parser = argparse.ArgumentParser(description=__doc__ + ' Requires Python 3.11 or newer.')
    parser.add_argument('command', nargs='?', choices=COMMANDS)
    parser.add_argument('options', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.command:
        dispatch(args.command, args.options)
        return
    if not sys.stdin.isatty():
        parser.error('The menu needs a terminal. Supply a command, for example: preflight.')
    entries = list(COMMANDS)
    while True:
        print('\nUlanziDeck · Umberto Giacobbi\n')
        for number, name in enumerate(entries, 1):
            print(f'  {number:2}  {name:12} {COMMANDS[name]}')
        print('   0  Close console')
        choice = input('\nChoose a number: ').strip()
        if choice in ('', '0'):
            print('Console closed.')
            return
        if not choice.isdigit() or not 1 <= int(choice) <= len(entries):
            print('Choose one of the listed numbers.')
            continue
        command = entries[int(choice) - 1]
        try:
            if command in ('setup', 'package', 'atlases', 'obs-guide', 'notices'):
                print('Enter command options. Use --help to list them; leave empty to return.')
                options = input('Options: ').strip()
                if options:
                    dispatch(command, shlex.split(options, posix=platform.system() != 'Windows'))
            else:
                dispatch(command, [])
        except (RuntimeError, ValueError, subprocess.CalledProcessError, OSError) as error:
            print(f'Action stopped. {error}', file=sys.stderr)
        input('\nPress Enter to return to the menu.')


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, ValueError, subprocess.CalledProcessError, OSError) as error:
        print(f'Action stopped. {error}', file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        print('\nConsole closed.', file=sys.stderr)
        sys.exit(130)
