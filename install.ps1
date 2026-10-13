$ErrorActionPreference = "Stop"

$Repo = "Soyagvs/shelly"
$Version = if ($env:SHELLY_VERSION) { $env:SHELLY_VERSION } else { "latest" }
$InstallDir = if ($env:SHELLY_INSTALL_DIR) { $env:SHELLY_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Shelly\bin" }
$Archive = "shelly-windows-x86_64.zip"

function Fail($Message) {
    Write-Error $Message
    exit 1
}

if (-not [Environment]::Is64BitOperatingSystem) {
    Fail "Shelly currently supports 64-bit Windows only."
}

if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/$Archive"
} else {
    if ($Version.StartsWith("v")) {
        $Tag = $Version
    } else {
        $Tag = "v$Version"
    }
    $Url = "https://github.com/$Repo/releases/download/$Tag/$Archive"
}

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("shelly-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir $Archive
    Write-Host "Downloading Shelly from $Url"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    $ExePath = Join-Path $TempDir "shelly.exe"
    if (-not (Test-Path $ExePath)) {
        Fail "Downloaded archive did not contain shelly.exe."
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path $ExePath -Destination (Join-Path $InstallDir "shelly.exe") -Force

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $PathParts = @()
    if ($UserPath) {
        $PathParts = $UserPath -split ";" | Where-Object { $_ -ne "" }
    }

    $AlreadyInPath = $false
    foreach ($Part in $PathParts) {
        if ($Part.TrimEnd("\") -ieq $InstallDir.TrimEnd("\")) {
            $AlreadyInPath = $true
            break
        }
    }

    if (-not $AlreadyInPath) {
        $NewPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "Added $InstallDir to your user PATH. Restart your terminal if 'shelly' is not found."
    }

    Write-Host "Shelly installed to $(Join-Path $InstallDir 'shelly.exe')"
    Write-Host "Run: shelly hello.shelly"
} finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
