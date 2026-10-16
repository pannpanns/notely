param([Parameter(Mandatory=$true)][string]$Installer)
$ErrorActionPreference = 'Stop'
$installRoot = Join-Path $env:RUNNER_TEMP ('Notely-install-test-' + [guid]::NewGuid().ToString('N'))
$installTimer = [Diagnostics.Stopwatch]::StartNew()
$setup = Start-Process -FilePath (Resolve-Path $Installer) -ArgumentList @('/S', "/D=$installRoot") -Wait -PassThru
if ($setup.ExitCode -ne 0) { throw "Installer failed: $($setup.ExitCode)" }
$installTimer.Stop()
New-Item -ItemType Directory -Force build_reports | Out-Null
"Install duration: $($installTimer.Elapsed.TotalSeconds) seconds" | Out-File -Encoding utf8 build_reports/install-duration.txt
$engineExe = Join-Path $installRoot 'notely-engine.exe'
if (-not (Test-Path (Join-Path $installRoot 'python311.dll'))) {
    throw 'Installed Python runtime not found beside engine.'
}
& ./.build-venv/Scripts/python.exe scripts/smoke_engine.py $engineExe
if ($LASTEXITCODE -ne 0) { throw 'Installed engine smoke test failed.' }
# Leave an actual orphan running and invoke the real uninstaller to verify hooks.
$listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$listener.Start()
$port = $listener.LocalEndpoint.Port
$listener.Stop()
$token = [guid]::NewGuid().ToString('N')
$dataRoot = Join-Path $env:RUNNER_TEMP ('Notely-data-test-' + $token)
$engine = Start-Process -FilePath $engineExe -ArgumentList @('--port', "$port", '--token', $token, '--data-root', $dataRoot) -PassThru
try {
    $ready = $false
    for ($i=0; $i -lt 60; $i++) {
        if ($engine.HasExited) { throw 'Engine exited before uninstall test.' }
        try {
            $null = Invoke-RestMethod "http://127.0.0.1:$port/models" -Headers @{Authorization="Bearer $token"} -TimeoutSec 2
            $ready = $true
            break
        } catch { Start-Sleep -Milliseconds 500 }
    }
    if (-not $ready) { throw 'Engine did not become ready.' }
    $keptModelRoot = Join-Path $installRoot 'NotelyModels'
    New-Item -ItemType Directory -Force $keptModelRoot | Out-Null
    [IO.File]::WriteAllText((Join-Path $keptModelRoot '.notely-owned'), 'app.notely.desktop')
    [IO.File]::WriteAllText((Join-Path $keptModelRoot 'keep-test.bin'), 'keep-model-test')
    $desktop = Start-Process -FilePath (Join-Path $installRoot 'notely-desktop.exe') -PassThru
    Start-Sleep -Seconds 3
    $uninstaller = Start-Process -FilePath (Join-Path $installRoot 'uninstall.exe') -ArgumentList '/S' -PassThru -Wait
    Write-Output "Uninstaller exit code: $($uninstaller.ExitCode)"
    if ($uninstaller.ExitCode -ne 0) { throw "Uninstaller failed: $($uninstaller.ExitCode). See cleanup log." }
    # NSIS can relaunch a temporary copy; wait for observable cleanup as well.
    for ($i=0; $i -lt 60; $i++) {
        if ($engine.HasExited -and -not (Test-Path $engineExe) -and -not (Test-Path (Join-Path $installRoot 'python311.dll'))) { break }
        Start-Sleep -Milliseconds 500
    }
    if (-not $engine.HasExited) { throw 'Uninstall left the engine running.' }
    if (-not $desktop.HasExited) { throw 'Uninstall left the desktop running.' }
    if ((Test-Path $engineExe) -or (Test-Path (Join-Path $installRoot 'python311.dll'))) {
        throw 'Uninstall left engine runtime files behind.'
    }
    if ([IO.File]::ReadAllText((Join-Path $keptModelRoot 'keep-test.bin')) -ne 'keep-model-test') { throw 'Uninstall did not preserve model data.' }
    Write-Output 'Installed layout, orphan shutdown and real uninstall: PASS'
} finally {
    New-Item -ItemType Directory -Force build_reports | Out-Null
    $cleanupLog = Join-Path $env:TEMP 'Notely-cleanup.log'
    if (Test-Path -LiteralPath $cleanupLog) {
        Copy-Item -LiteralPath $cleanupLog -Destination build_reports/uninstall-cleanup.log -Force
        Get-Content -LiteralPath $cleanupLog | Write-Output
    }
    if ($desktop -and -not $desktop.HasExited) { $desktop.Kill() }
    if (-not $engine.HasExited) { $engine.Kill(); $null = $engine.WaitForExit(10000) }
}
