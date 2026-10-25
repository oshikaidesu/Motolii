[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string[]]$DefinitionPath,

    [ValidatePattern('^\d+(?:\.\d+)?$')]
    [string]$Speed = '1',

    [switch]$PreserveExistingSpeed,

    [string]$BackupBase = '',

    [switch]$Recurse
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$scriptDirectory = [IO.Path]::GetDirectoryName($PSCommandPath)
$checksumHelper = Join-Path $scriptDirectory 'PsdToolKitChecksum.ps1'
if (-not (Test-Path -LiteralPath $checksumHelper -PathType Leaf)) {
    throw "チェックサムヘルパーがありません: $checksumHelper"
}
. $checksumHelper

$acceptedScripts = @(
    'PSDToolKit.LipSync',
    'PSDToolKit.LipSyncAviUtl1',
    'LipSyncAviUtl1',
    'LipSyncAviUtl1Legacy',
    'LipSyncSyllablePulse'
)
$metadataPattern = '(?s)--\[==\[PTK:(\{.*?\})\]==\]'
$utf8NoBom = [Text.UTF8Encoding]::new($false)

function Get-NormalizedDefinitionPaths {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$InputPath,

        [Parameter(Mandatory = $true)]
        [bool]$SearchRecursively
    )

    $paths = [Collections.Generic.HashSet[string]]::new(
        [StringComparer]::OrdinalIgnoreCase
    )
    foreach ($input in $InputPath) {
        if (-not (Test-Path -LiteralPath $input)) {
            throw "定義ファイルまたはディレクトリがありません: $input"
        }

        $item = Get-Item -LiteralPath $input
        if ($item.PSIsContainer) {
            $childArguments = @{
                LiteralPath = $item.FullName
                File = $true
            }
            if ($SearchRecursively) {
                $childArguments.Recurse = $true
            }
            foreach ($child in Get-ChildItem @childArguments) {
                if ($child.Extension -in @('.anm2', '.obj2')) {
                    [void]$paths.Add($child.FullName)
                }
            }
            continue
        }

        if ($item.Extension -notin @('.anm2', '.obj2')) {
            throw "対応していない拡張子です: $($item.FullName)"
        }
        [void]$paths.Add($item.FullName)

        $pairedExtension = if ($item.Extension -eq '.anm2') { '.obj2' } else { '.anm2' }
        $pairedPath = [IO.Path]::ChangeExtension($item.FullName, $pairedExtension)
        if (-not (Test-Path -LiteralPath $pairedPath -PathType Leaf)) {
            throw "ANM2/OBJ2の対応ファイルがありません: $pairedPath"
        }
        [void]$paths.Add([IO.Path]::GetFullPath($pairedPath))
    }

    return @($paths | Sort-Object)
}

function Get-LipSyncSnapshot {
    param(
        [Parameter(Mandatory = $true)]
        $Metadata,

        [switch]$ExcludeManagedSettings
    )

    $snapshot = [Collections.Generic.List[object]]::new()
    foreach ($selector in $Metadata.selectors) {
        foreach ($item in $selector.items) {
            if ($item -isnot [Management.Automation.PSCustomObject] -or
                $item.script -notin $acceptedScripts) {
                continue
            }

            $parameters = [Collections.Generic.List[object]]::new()
            foreach ($parameter in $item.params) {
                if ($ExcludeManagedSettings -and
                    $parameter[0] -in @('速さ', '発声がなくても有効')) {
                    continue
                }
                $parameters.Add(@([string]$parameter[0], [string]$parameter[1]))
            }
            $snapshot.Add([ordered]@{
                Group = [string]$selector.group
                Name = [string]$item.n
                Script = if ($ExcludeManagedSettings) { $null } else { [string]$item.script }
                Parameters = $parameters.ToArray()
            })
        }
    }
    return $snapshot.ToArray()
}

function Get-DefinitionRecord {
    param([Parameter(Mandatory = $true)][string]$Path)

    $content = [IO.File]::ReadAllText($Path, [Text.Encoding]::UTF8)
    $metadataMatch = [regex]::Match($content, $metadataPattern)
    if (-not $metadataMatch.Success) {
        return [pscustomobject]@{
            Path = $Path
            Content = $content
            Metadata = $null
            LipSyncItems = 0
        }
    }

    $metadata = $metadataMatch.Groups[1].Value | ConvertFrom-Json
    $snapshot = @(Get-LipSyncSnapshot -Metadata $metadata)
    return [pscustomobject]@{
        Path = $Path
        Content = $content
        Metadata = $metadata
        LipSyncItems = $snapshot.Count
    }
}

function Convert-DefinitionRecord {
    param(
        [Parameter(Mandatory = $true)]
        $Record,

        [Parameter(Mandatory = $true)]
        [string]$AnimationSpeed,

        [Parameter(Mandatory = $true)]
        [bool]$KeepExistingSpeed
    )

    $beforeSnapshot = @(
        Get-LipSyncSnapshot -Metadata $Record.Metadata -ExcludeManagedSettings
    ) | ConvertTo-Json -Depth 100 -Compress

    $updatedItems = 0
    $resolvedSpeeds = [Collections.Generic.List[string]]::new()
    foreach ($selector in $Record.Metadata.selectors) {
        foreach ($item in $selector.items) {
            if ($item -isnot [Management.Automation.PSCustomObject] -or
                $item.script -notin $acceptedScripts) {
                continue
            }

            $item.script = 'LipSyncAviUtl1'
            $existingSpeed = ''
            foreach ($parameter in $item.params) {
                if ($parameter[0] -eq '速さ') {
                    $existingSpeed = [string]$parameter[1]
                    break
                }
            }
            $itemSpeed = if ($KeepExistingSpeed -and $existingSpeed -ne '') {
                $existingSpeed
            }
            else {
                $AnimationSpeed
            }
            $parameters = [Collections.Generic.List[object]]::new()
            $insertedSpeed = $false
            $insertedAlwaysApply = $false
            foreach ($parameter in $item.params) {
                if ($parameter[0] -eq '速さ') {
                    continue
                }
                if ($parameter[0] -eq '発声がなくても有効' -and -not $insertedSpeed) {
                    $parameters.Add(@('速さ', $itemSpeed))
                    $insertedSpeed = $true
                }
                if ($parameter[0] -eq '発声がなくても有効') {
                    $parameters.Add(@('発声がなくても有効', '1'))
                    $insertedAlwaysApply = $true
                    continue
                }
                $parameters.Add(@([string]$parameter[0], [string]$parameter[1]))
            }
            if (-not $insertedSpeed) {
                $parameters.Add(@('速さ', $itemSpeed))
            }
            if (-not $insertedAlwaysApply) {
                $parameters.Add(@('発声がなくても有効', '1'))
            }
            $item.params = $parameters.ToArray()
            $resolvedSpeeds.Add($itemSpeed)
            $updatedItems++
        }
    }

    $metadataJson = $Record.Metadata | ConvertTo-Json -Depth 100 -Compress
    $metadataMatch = [regex]::Match($Record.Content, $metadataPattern)
    $updatedContent =
        $Record.Content.Substring(0, $metadataMatch.Index) +
        "--[==[PTK:$metadataJson]==]" +
        $Record.Content.Substring($metadataMatch.Index + $metadataMatch.Length)

    if ([IO.Path]::GetExtension($Record.Path) -eq '.anm2') {
        $updatedContent = [regex]::Replace(
            $updatedContent,
            '(?m)^--check@syllable_pulse:音節パルス,0\r?\n',
            ''
        )
        $updatedContent = $updatedContent.Replace(
            '(syllable_pulse ~= 0 and require("LipSyncSyllablePulse") or require("LipSyncAviUtl1")).new(',
            'require("LipSyncAviUtl1").new('
        )
        $updatedContent = $updatedContent -replace (
            'require\("(?:PSDToolKit\.LipSync(?:AviUtl1)?|LipSyncAviUtl1(?:Legacy)?|LipSyncSyllablePulse)"\)\.new\('
        ), 'require("LipSyncAviUtl1").new('
        $updatedContent = [regex]::Replace(
            $updatedContent,
            '(?m)^\s*\["速さ"\]\s*=\s*"[^"]+",\r?\n',
            ''
        )
        $speedState = [pscustomobject]@{ Index = 0 }
        $updatedContent = [regex]::Replace(
            $updatedContent,
            '(?m)^(?<line>(?<indent>\s*)\["感度"\]\s*=\s*"[^"]+",\r?\n)',
            [Text.RegularExpressions.MatchEvaluator]{
                param($match)

                if ($speedState.Index -ge $resolvedSpeeds.Count) {
                    throw "Luaの感度設定数が口パク項目数を超えています: $($Record.Path)"
                }
                $resolvedSpeed = $resolvedSpeeds[$speedState.Index]
                $speedState.Index++
                return $match.Groups['line'].Value +
                    $match.Groups['indent'].Value +
                    "[`"速さ`"] = `"$resolvedSpeed`",`r`n"
            }
        )
        $updatedContent = [regex]::Replace(
            $updatedContent,
            '(?m)^(\s*\["発声がなくても有効"\]\s*=\s*)"[^"]+"(,\r?\n)',
            '$1"1"$2'
        )
        if ($speedState.Index -ne $updatedItems) {
            throw "Luaの感度設定数が口パク項目数と一致しません: $($Record.Path)"
        }
        $alwaysApplyCount = (
            [regex]::Matches(
                $updatedContent,
                '\["発声がなくても有効"\]\s*=\s*"1"'
            )
        ).Count
        if ($alwaysApplyCount -ne $updatedItems) {
            throw "Luaの無音時適用数が口パク項目数と一致しません: $($Record.Path)"
        }

        $plainConstructorCount = (
            [regex]::Matches($updatedContent, 'require\("LipSyncAviUtl1"\)\.new\(')
        ).Count
        if ($plainConstructorCount -ne $updatedItems) {
            throw "Luaコンストラクター数が一致しません: $($Record.Path)"
        }
        if ($updatedContent -match '音節パルス|syllable_pulse') {
            throw "旧音節パルス切替設定が残っています: $($Record.Path)"
        }
        $luaSpeeds = @(
            [regex]::Matches($updatedContent, '\["速さ"\]\s*=\s*"([^"]+)"') |
                ForEach-Object { $_.Groups[1].Value }
        )
        if ($luaSpeeds.Count -ne $updatedItems) {
            throw "Luaの速さ設定数が一致しません: $($Record.Path)"
        }
        for ($index = 0; $index -lt $luaSpeeds.Count; $index++) {
            if ($luaSpeeds[$index] -cne $resolvedSpeeds[$index]) {
                throw "Luaの速さ設定値がメタデータと一致しません: $($Record.Path)"
            }
        }
    }

    $updatedContent = [regex]::Replace($updatedContent, '\r?\n', "`r`n")
    $checksumResult = Update-PsdToolKitDefinitionChecksum `
        -Content $updatedContent `
        -Extension ([IO.Path]::GetExtension($Record.Path))
    $updatedContent = $checksumResult.Content
    $afterMetadataMatch = [regex]::Match($updatedContent, $metadataPattern)
    $afterMetadata = $afterMetadataMatch.Groups[1].Value | ConvertFrom-Json
    $afterSnapshot = @(
        Get-LipSyncSnapshot -Metadata $afterMetadata -ExcludeManagedSettings
    ) | ConvertTo-Json -Depth 100 -Compress
    if ($beforeSnapshot -cne $afterSnapshot) {
        throw "口パクのパスまたは既存パラメーターが変化しました: $($Record.Path)"
    }

    return [pscustomobject]@{
        Path = $Record.Path
        Content = $updatedContent
        LipSyncItems = $updatedItems
        Metadata = $afterMetadata
    }
}

$resolvedPaths = @(Get-NormalizedDefinitionPaths `
    -InputPath $DefinitionPath `
    -SearchRecursively $Recurse.IsPresent)
$records = @($resolvedPaths | ForEach-Object { Get-DefinitionRecord -Path $_ })
$targetRecords = @($records | Where-Object { $_.LipSyncItems -gt 0 })
if ($targetRecords.Count -eq 0) {
    throw '口パク定義を含むANM2/OBJ2が見つかりませんでした。'
}

$groups = $targetRecords | Group-Object {
    [IO.Path]::Combine(
        [IO.Path]::GetDirectoryName($_.Path),
        [IO.Path]::GetFileNameWithoutExtension($_.Path)
    )
}
foreach ($group in $groups) {
    $extensions = @($group.Group | ForEach-Object { [IO.Path]::GetExtension($_.Path) })
    if ($extensions.Count -ne 2 -or
        '.anm2' -notin $extensions -or
        '.obj2' -notin $extensions) {
        throw "ANM2/OBJ2の口パク定義が対になっていません: $($group.Name)"
    }
}

$converted = @($targetRecords | ForEach-Object {
    Convert-DefinitionRecord `
        -Record $_ `
        -AnimationSpeed $Speed `
        -KeepExistingSpeed $PreserveExistingSpeed.IsPresent
})
foreach ($group in ($converted | Group-Object {
    [IO.Path]::Combine(
        [IO.Path]::GetDirectoryName($_.Path),
        [IO.Path]::GetFileNameWithoutExtension($_.Path)
    )
})) {
    $snapshots = @($group.Group | ForEach-Object {
        @(Get-LipSyncSnapshot -Metadata $_.Metadata) |
            ConvertTo-Json -Depth 100 -Compress
    } | Select-Object -Unique)
    if ($snapshots.Count -ne 1) {
        throw "ANM2/OBJ2の変換後メタデータが一致しません: $($group.Name)"
    }
}

if ($BackupBase -eq '') {
    $firstDirectory = [IO.Path]::GetDirectoryName($targetRecords[0].Path)
    if ($firstDirectory.StartsWith(
        'C:\ProgramData\aviutl2',
        [StringComparison]::OrdinalIgnoreCase
    )) {
        $BackupBase = 'C:\ProgramData\aviutl2\Backup'
    }
    else {
        $BackupBase = Join-Path $firstDirectory '.backup'
    }
}

$timestamp = Get-Date -Format 'yyyyMMdd_HHmmss_fff'
$backupRoot = Join-Path $BackupBase "PSDToolKit_NaturalLipSync_Definition_$timestamp"
New-Item -ItemType Directory -Path $backupRoot -Force | Out-Null

$beforeEntries = [Collections.Generic.List[object]]::new()
foreach ($record in $targetRecords) {
    $backupPath = Join-Path $backupRoot ([IO.Path]::GetFileName($record.Path))
    if (Test-Path -LiteralPath $backupPath) {
        throw "バックアップ名が重複しています: $backupPath"
    }
    Copy-Item -LiteralPath $record.Path -Destination $backupPath
    $beforeEntries.Add([ordered]@{
        Path = $record.Path
        BackupPath = $backupPath
        Sha256 = (Get-FileHash -LiteralPath $record.Path -Algorithm SHA256).Hash
        LipSyncItems = $record.LipSyncItems
    })
}

$updatedEntries = [Collections.Generic.List[object]]::new()
foreach ($definition in $converted) {
    [IO.File]::WriteAllText($definition.Path, $definition.Content, $utf8NoBom)
    $updatedEntries.Add([ordered]@{
        Path = $definition.Path
        Sha256 = (Get-FileHash -LiteralPath $definition.Path -Algorithm SHA256).Hash
        LipSyncItems = $definition.LipSyncItems
        Speed = $Speed
    })
}

$manifest = [ordered]@{
    CreatedAt = (Get-Date).ToString('o')
    BackupRoot = $backupRoot
    Speed = $Speed
    PreserveExistingSpeed = $PreserveExistingSpeed.IsPresent
    Files = $updatedEntries.Count
    Definitions = $updatedEntries.Count / 2
    Before = $beforeEntries.ToArray()
    Updated = $updatedEntries.ToArray()
}
$manifestPath = Join-Path $backupRoot 'manifest.json'
[IO.File]::WriteAllText(
    $manifestPath,
    (($manifest | ConvertTo-Json -Depth 8) + "`r`n"),
    $utf8NoBom
)

$updatedLipSyncItems = 0
foreach ($entry in $updatedEntries.ToArray()) {
    $updatedLipSyncItems += [int]$entry['LipSyncItems']
}

[pscustomobject]@{
    BackupRoot = $backupRoot
    Manifest = $manifestPath
    UpdatedFiles = $updatedEntries.Count
    UpdatedDefinitions = $updatedEntries.Count / 2
    UpdatedLipSyncItems = $updatedLipSyncItems
    Speed = $Speed
    PreserveExistingSpeed = $PreserveExistingSpeed.IsPresent
}
