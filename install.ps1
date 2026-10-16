$ErrorActionPreference = "Stop"

$Repo = "Soyagvs/shelly"
$Version = if ($env:SHELLY_VERSION) { $env:SHELLY_VERSION } else { "latest" }
$InstallDir = if ($env:SHELLY_INSTALL_DIR) { $env:SHELLY_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Shelly\bin" }
$Archive = "shelly-windows-x86_64.zip"


function Install-ShellyFileAssociation($InstallDir, $IconUrl) {
    $IconPath = Join-Path $InstallDir "shelly.ico"
    try {
        Invoke-WebRequest -Uri $IconUrl -OutFile $IconPath
    } catch {
        Write-Host "Could not download Shelly file icon; continuing without icon registration."
        return
    }

    $ClassesRoot = "HKCU:\Software\Classes"
    $ExtensionKey = Join-Path $ClassesRoot ".shelly"
    $ProgId = "ShellyScript"
    $ProgIdKey = Join-Path $ClassesRoot $ProgId

    New-Item -Path $ExtensionKey -Force | Out-Null
    Set-Item -Path $ExtensionKey -Value $ProgId
    New-ItemProperty -Path $ExtensionKey -Name "Content Type" -Value "text/x-shelly" -PropertyType String -Force | Out-Null
    New-ItemProperty -Path $ExtensionKey -Name "PerceivedType" -Value "text" -PropertyType String -Force | Out-Null

    New-Item -Path $ProgIdKey -Force | Out-Null
    Set-Item -Path $ProgIdKey -Value "Shelly script"

    $DefaultIconKey = Join-Path $ProgIdKey "DefaultIcon"
    New-Item -Path $DefaultIconKey -Force | Out-Null
    Set-Item -Path $DefaultIconKey -Value $IconPath

    $CommandKey = Join-Path $ProgIdKey "shell\open\command"
    New-Item -Path $CommandKey -Force | Out-Null
    $ExePath = Join-Path $InstallDir "shelly.exe"
    Set-Item -Path $CommandKey -Value "`"$ExePath`" `"%1`""

    try {
        Start-Process -FilePath "ie4uinit.exe" -ArgumentList "-show" -WindowStyle Hidden -ErrorAction SilentlyContinue
    } catch {
        # Explorer icon refresh is best-effort.
    }

    Write-Host "Registered .shelly file association and icon for the current user."
}

function Fail($Message) {
    Write-Error $Message
    exit 1
}

if (-not [Environment]::Is64BitOperatingSystem) {
    Fail "Shelly currently supports 64-bit Windows only."
}

if ($Version -eq "latest") {
    $Url = "https://github.com/$Repo/releases/latest/download/$Archive"
    $IconRef = "main"
} else {
    if ($Version.StartsWith("v")) {
        $Tag = $Version
    } else {
        $Tag = "v$Version"
    }
    $Url = "https://github.com/$Repo/releases/download/$Tag/$Archive"
    $IconRef = $Tag
}
$IconUrl = "https://raw.githubusercontent.com/$Repo/$IconRef/assets/icons/shelly.ico"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("shelly-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir $Archive
    Write-Host "Downloading Shelly from $Url"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    $ExePath = Join-Path $TempDir "shelly.exe"
    $LspExePath = Join-Path $TempDir "shelly-lsp.exe"
    if (-not (Test-Path $ExePath)) {
        Fail "Downloaded archive did not contain shelly.exe; cannot install the CLI."
    }
    if (-not (Test-Path $LspExePath)) {
        Fail "Downloaded archive did not contain shelly-lsp.exe; cannot install the language server."
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path $ExePath -Destination (Join-Path $InstallDir "shelly.exe") -Force
    Copy-Item -Path $LspExePath -Destination (Join-Path $InstallDir "shelly-lsp.exe") -Force

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
    Install-ShellyFileAssociation -InstallDir $InstallDir -IconUrl $IconUrl
    Write-Host "Run: shelly hello.shelly"
} finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
