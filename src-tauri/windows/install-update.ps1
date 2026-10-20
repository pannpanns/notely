param(
    [Parameter(Mandatory=$true)][string]$Installer,
    [Parameter(Mandatory=$true)][string]$InstallDir,
    [Parameter(Mandatory=$true)][int]$AppPid
)
$ErrorActionPreference = 'Stop'
try {
    # Wait for Notely and its engine to release installed files.
    Wait-Process -Id $AppPid -Timeout 60 -ErrorAction SilentlyContinue
    if (Get-Process -Id $AppPid -ErrorAction SilentlyContinue) { throw 'Notely masih berjalan.' }
    # NSIS requires /D to be the last argument, with no surrounding quotes.
    # No silent switch: the user sees and completes the installer.
    $process = Start-Process -FilePath $Installer -ArgumentList ('/D=' + $InstallDir) -PassThru -Wait
    if ($process.ExitCode -ne 0) { throw "Installer selesai dengan kode $($process.ExitCode)." }
} catch {
    Add-Type -AssemblyName System.Windows.Forms
    [System.Windows.Forms.MessageBox]::Show("Update belum selesai. Buka Notely dan coba lagi.`n$($_.Exception.Message)", 'Notely Update') | Out-Null
} finally {
    # Delete only this updater's exact download, including on cancellation.
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Remove-Item -LiteralPath $Installer -Force -ErrorAction SilentlyContinue
        if (-not (Test-Path -LiteralPath $Installer)) { break }
        Start-Sleep -Seconds 2
    }
    Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
}
