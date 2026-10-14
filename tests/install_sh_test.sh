#!/bin/sh
set -eu

# Isolated coverage for install.sh PATH setup. The installer is expected to
# add only the marked block below when running under Bash or Zsh:
#   # >>> shelly PATH >>>
#   export PATH="...:$PATH"
#   # <<< shelly PATH <<<

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TEST_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/shelly-install-test.XXXXXX")
trap 'rm -rf "$TEST_ROOT"' EXIT HUP INT TERM

MOCK_BIN=$TEST_ROOT/mock-bin
mkdir -p "$MOCK_BIN"

cat > "$MOCK_BIN/uname" <<'MOCK'
#!/bin/sh
case "$1" in
  -s) printf '%s\n' Linux ;;
  -m) printf '%s\n' x86_64 ;;
  *) exit 1 ;;
esac
MOCK

cat > "$MOCK_BIN/curl" <<'MOCK'
#!/bin/sh
# Return the local fixture for every download request, including icon requests.
output=
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) output=$2; shift 2 ;;
    *) shift ;;
  esac
done
[ -n "$output" ] || exit 1
cp "$SHELLY_TEST_ARCHIVE" "$output"
MOCK

chmod +x "$MOCK_BIN/uname" "$MOCK_BIN/curl"

PAYLOAD=$TEST_ROOT/payload
mkdir -p "$PAYLOAD"
printf '#!/bin/sh\nexit 0\n' > "$PAYLOAD/shelly"
chmod +x "$PAYLOAD/shelly"
ARCHIVE=$TEST_ROOT/shelly-linux-x86_64.tar.gz
tar -czf "$ARCHIVE" -C "$PAYLOAD" shelly

run_install() {
  HOME=$1 INSTALL_DIR=$2 SHELL=$3 PATH="$4" SHELLY_TEST_ARCHIVE="$ARCHIVE" \
    sh "$ROOT/install.sh" >"$5" 2>&1
}

assert_file_contains() {
  grep -F -- "$2" "$1" >/dev/null || {
    printf 'assertion failed: %s does not contain %s\n' "$1" "$2" >&2
    exit 1
  }
}

assert_same() {
  cmp -s "$1" "$2" || {
    printf 'assertion failed: %s changed\n' "$1" >&2
    exit 1
  }
}

# Bash creates .bashrc, appends one marked block, and remains idempotent.
BASH_HOME=$TEST_ROOT/bash-home
BASH_INSTALL=$TEST_ROOT/bash-install
mkdir -p "$BASH_HOME" "$BASH_INSTALL"
printf 'existing bash config\n' > "$BASH_HOME/.bashrc"
run_install "$BASH_HOME" "$BASH_INSTALL" /bin/bash "$MOCK_BIN:/usr/bin:/bin" "$TEST_ROOT/bash-first.log"
assert_file_contains "$BASH_HOME/.bashrc" '# >>> shelly PATH >>>'
assert_file_contains "$BASH_HOME/.bashrc" "export PATH=\"$BASH_INSTALL:\$PATH\""
assert_file_contains "$BASH_HOME/.bashrc" '# <<< shelly PATH <<<'
first_bashrc=$TEST_ROOT/bashrc.first
cp "$BASH_HOME/.bashrc" "$first_bashrc"
run_install "$BASH_HOME" "$BASH_INSTALL" /bin/bash "$MOCK_BIN:/usr/bin:/bin" "$TEST_ROOT/bash-second.log"
assert_same "$BASH_HOME/.bashrc" "$first_bashrc"

# An install directory already on PATH does not touch any profile.
ON_PATH_HOME=$TEST_ROOT/on-path-home
ON_PATH_INSTALL=$TEST_ROOT/on-path-install
mkdir -p "$ON_PATH_HOME" "$ON_PATH_INSTALL"
printf 'unchanged\n' > "$ON_PATH_HOME/.bashrc"
cp "$ON_PATH_HOME/.bashrc" "$TEST_ROOT/on-path.before"
run_install "$ON_PATH_HOME" "$ON_PATH_INSTALL" /bin/bash "$MOCK_BIN:$ON_PATH_INSTALL:/usr/bin:/bin" "$TEST_ROOT/on-path.log"
assert_same "$ON_PATH_HOME/.bashrc" "$TEST_ROOT/on-path.before"

# Zsh creates .zshrc, appends the same marked block, and remains idempotent.
ZSH_HOME=$TEST_ROOT/zsh-home
ZSH_INSTALL=$TEST_ROOT/zsh-install
mkdir -p "$ZSH_HOME" "$ZSH_INSTALL"
printf 'existing zsh config\n' > "$ZSH_HOME/.zshrc"
run_install "$ZSH_HOME" "$ZSH_INSTALL" /bin/zsh "$MOCK_BIN:/usr/bin:/bin" "$TEST_ROOT/zsh-first.log"
assert_file_contains "$ZSH_HOME/.zshrc" '# >>> shelly PATH >>>'
assert_file_contains "$ZSH_HOME/.zshrc" "export PATH=\"$ZSH_INSTALL:\$PATH\""
assert_file_contains "$ZSH_HOME/.zshrc" '# <<< shelly PATH <<<'
first_zshrc=$TEST_ROOT/zshrc.first
cp "$ZSH_HOME/.zshrc" "$first_zshrc"
run_install "$ZSH_HOME" "$ZSH_INSTALL" /bin/zsh "$MOCK_BIN:/usr/bin:/bin" "$TEST_ROOT/zsh-second.log"
assert_same "$ZSH_HOME/.zshrc" "$first_zshrc"

# Unknown and Fish shells leave profiles unchanged and provide their actual fallback guidance.
for shell_name in /bin/fish /bin/ksh; do
  shell_base=$(basename "$shell_name")
  shell_home=$TEST_ROOT/$shell_base-home
  shell_install=$TEST_ROOT/$shell_base-install
  mkdir -p "$shell_home" "$shell_install"
  printf 'profile\n' > "$shell_home/.bashrc"
  cp "$shell_home/.bashrc" "$TEST_ROOT/$shell_base.before"
  run_install "$shell_home" "$shell_install" "$shell_name" "$MOCK_BIN:/usr/bin:/bin" "$TEST_ROOT/$shell_base.log"
  assert_same "$shell_home/.bashrc" "$TEST_ROOT/$shell_base.before"
done
assert_file_contains "$TEST_ROOT/fish.log" 'Could not configure PATH automatically for Fish. Add Shelly with:'
assert_file_contains "$TEST_ROOT/fish.log" "fish_add_path \"$TEST_ROOT/fish-install\""
assert_file_contains "$TEST_ROOT/ksh.log" 'Could not determine a supported shell profile. Add Shelly to PATH manually:'
assert_file_contains "$TEST_ROOT/ksh.log" "export PATH=\"$TEST_ROOT/ksh-install:\$PATH\""

printf '%s\n' 'install.sh PATH setup tests passed'
