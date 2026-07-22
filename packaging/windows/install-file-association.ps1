param(
	[string]$InstallDirectory = (Split-Path -Parent $PSScriptRoot)
)

$Executable = Join-Path $InstallDirectory "Aedicule.exe"
$ApplicationIcon = Join-Path $InstallDirectory "Aedicule.ico"
$DocumentIcon = Join-Path $InstallDirectory "Aedicule-WAT.ico"
$Classes = "HKCU:\Software\Classes"
$ProgId = "Aedicule.WAT"

New-Item -Force "$Classes\.wat" | Out-Null
Set-Item -Path "$Classes\.wat" -Value $ProgId
New-Item -Force "$Classes\$ProgId" | Out-Null
Set-Item -Path "$Classes\$ProgId" -Value "WebAssembly Text application"
New-Item -Force "$Classes\$ProgId\DefaultIcon" | Out-Null
Set-Item -Path "$Classes\$ProgId\DefaultIcon" -Value ('"' + $DocumentIcon + '",0')
New-Item -Force "$Classes\$ProgId\shell\open\command" | Out-Null
Set-Item -Path "$Classes\$ProgId\shell\open\command" -Value ('"' + $Executable + '" "%1"')
New-Item -Force "$Classes\Applications\Aedicule.exe\DefaultIcon" | Out-Null
Set-Item -Path "$Classes\Applications\Aedicule.exe\DefaultIcon" -Value ('"' + $ApplicationIcon + '",0')

Write-Host "Aedicule now opens .wat documents for this user."
