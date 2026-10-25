[CmdletBinding(DefaultParameterSetName = 'Paths')]
param(
    [Parameter(
        Mandatory = $true,
        Position = 0,
        ValueFromRemainingArguments = $true,
        ParameterSetName = 'Paths'
    )]
    [string[]]$DroppedPath,

    [Parameter(Mandatory = $true, ParameterSetName = 'Encoded')]
    [string]$EncodedRequest,

    [switch]$Elevated
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$scriptDirectory = [IO.Path]::GetDirectoryName($PSCommandPath)
$converterPath = Join-Path $scriptDirectory 'Convert-PSDToolKitLipSyncDefinition.ps1'
$runtimeDirectory = 'C:\ProgramData\aviutl2\Script\PSDToolKit'
$runtimeLua = Join-Path $runtimeDirectory 'LipSyncAviUtl1.lua'
$runtimeNative = Join-Path $runtimeDirectory 'LipSyncAviUtl1.mod2'
$animationSpeed = '1'

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator
    )
}

function Convert-RequestToBase64 {
    param([Parameter(Mandatory = $true)][string[]]$Paths)

    $json = [ordered]@{ Paths = $Paths } | ConvertTo-Json -Compress
    return [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($json))
}

function Convert-Base64ToRequestPaths {
    param([Parameter(Mandatory = $true)][string]$Value)

    try {
        $json = [Text.Encoding]::UTF8.GetString(
            [Convert]::FromBase64String($Value)
        )
        $request = $json | ConvertFrom-Json
    }
    catch {
        throw '管理者プロセスへ渡された変換要求を読み取れませんでした。'
    }
    return @($request.Paths | ForEach-Object { [string]$_ })
}

function Get-NormalizedDroppedPaths {
    param([Parameter(Mandatory = $true)][string[]]$Paths)

    $normalized = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($path in $Paths) {
        if ([string]::IsNullOrWhiteSpace($path)) {
            continue
        }
        if (-not (Test-Path -LiteralPath $path)) {
            throw "ドロップされたパスがありません: $path"
        }

        $item = Get-Item -LiteralPath $path
        if (-not $item.PSIsContainer -and $item.Extension -notin @('.anm2', '.obj2')) {
            if ($item.Extension -eq '.psd') {
                throw 'PSDファイルから口レイヤーを自動判定できません。PSDToolKitで定義を生成し、.anm2/.obj2をドロップしてください。'
            }
            throw "対応していないファイルです: $($item.FullName)"
        }
        [void]$normalized.Add($item.FullName)
    }

    if ($normalized.Count -eq 0) {
        throw '変換対象が指定されていません。'
    }
    return @($normalized | Sort-Object)
}

try {
    if (-not (Test-Path -LiteralPath $converterPath -PathType Leaf)) {
        throw "変換器がありません: $converterPath"
    }
    foreach ($runtimePath in @($runtimeLua, $runtimeNative)) {
        if (-not (Test-Path -LiteralPath $runtimePath -PathType Leaf)) {
            throw "AviUtl1互換ランタイムが未導入です: $runtimePath"
        }
    }

    $requestPaths = if ($PSCmdlet.ParameterSetName -eq 'Encoded') {
        Convert-Base64ToRequestPaths -Value $EncodedRequest
    }
    else {
        $DroppedPath
    }
    $normalizedPaths = @(Get-NormalizedDroppedPaths -Paths $requestPaths)

    $requiresAdministrator = $false
    foreach ($path in $normalizedPaths) {
        $fullPath = [IO.Path]::GetFullPath($path)
        if ($fullPath.StartsWith(
            $runtimeDirectory,
            [StringComparison]::OrdinalIgnoreCase
        )) {
            $requiresAdministrator = $true
            break
        }
    }

    if ($requiresAdministrator -and -not (Test-IsAdministrator)) {
        if ($Elevated) {
            throw '管理者権限で起動しましたが、管理者トークンを確認できません。'
        }

        Write-Host 'C:\ProgramDataへの書き込み権限を取得します。' -ForegroundColor Cyan
        $encoded = Convert-RequestToBase64 -Paths $normalizedPaths
        $powershellPath = Join-Path $PSHOME 'powershell.exe'
        $quotedScriptPath = '"' + $PSCommandPath.Replace('"', '\"') + '"'
        $process = Start-Process `
            -FilePath $powershellPath `
            -Verb RunAs `
            -ArgumentList @(
                '-NoLogo',
                '-NoProfile',
                '-ExecutionPolicy',
                'Bypass',
                '-File',
                $quotedScriptPath,
                '-EncodedRequest',
                $encoded,
                '-Elevated'
            ) `
            -Wait `
            -PassThru
        exit $process.ExitCode
    }

    Write-Host 'PSDToolKit定義をAviUtl1互換口パクへ変換します。' -ForegroundColor Cyan
    Write-Host "ドロップ項目: $($normalizedPaths.Count)"
    foreach ($path in $normalizedPaths) {
        Write-Host "  $path"
    }
    $result = & $converterPath `
        -DefinitionPath $normalizedPaths `
        -Speed $animationSpeed `
        -PreserveExistingSpeed

    Write-Host ''
    Write-Host '変換と検証が完了しました。' -ForegroundColor Green
    Write-Host '更新方式: 元のANM2/OBJ2をその場で上書き'
    Write-Host "更新ファイル: $($result.UpdatedFiles)"
    Write-Host "口パク項目: $($result.UpdatedLipSyncItems)"
    Write-Host '既存の速さ: 保持（未設定時は1）'
    Write-Host "バックアップ: $($result.BackupRoot)"
    Write-Host '別フォルダへ出力されるのはバックアップだけです。'
    Write-Host 'AviUtl2を再起動してから定義を使用してください。'
    Write-Output $result
}
catch {
    Write-Host ''
    Write-Host '変換に失敗しました。' -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}
