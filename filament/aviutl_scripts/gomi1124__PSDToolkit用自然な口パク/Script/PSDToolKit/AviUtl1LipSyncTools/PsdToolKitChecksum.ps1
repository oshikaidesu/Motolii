if (-not ('PsdToolKitDefinitionCyrb64' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Text;

public static class PsdToolKitDefinitionCyrb64
{
    public static string ComputeHex(string value)
    {
        byte[] data = new UTF8Encoding(false).GetBytes(value);
        int wordCount = (data.Length + 3) / 4;
        byte[] padded = new byte[wordCount * 4];
        Buffer.BlockCopy(data, 0, padded, 0, data.Length);
        uint h1 = 0x91eb9dc7U;
        uint h2 = 0x41c6ce57U;

        unchecked
        {
            for (int index = 0; index < wordCount; index++)
            {
                uint word = BitConverter.ToUInt32(padded, index * 4);
                h1 = (h1 ^ word) * 2654435761U;
                h2 = (h2 ^ word) * 1597334677U;
            }
            h1 = ((h1 ^ (h1 >> 16)) * 2246822507U) ^
                 ((h2 ^ (h2 >> 13)) * 3266489909U);
            h2 = ((h2 ^ (h2 >> 16)) * 2246822507U) ^
                 ((h1 ^ (h1 >> 13)) * 3266489909U);
        }

        ulong result = ((ulong)h2 << 32) | h1;
        return result.ToString("x16");
    }
}
'@
}

function Update-PsdToolKitDefinitionChecksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content,

        [Parameter(Mandatory = $true)]
        [ValidateSet('.anm2', '.obj2')]
        [string]$Extension
    )

    $metadataPattern = '(?s)--\[==\[PTK:(\{.*?\})\]==\]'
    $metadataMatch = [regex]::Match($Content, $metadataPattern)
    if (-not $metadataMatch.Success) {
        throw 'PTKメタデータがないためチェックサムを更新できません。'
    }

    $metadata = $metadataMatch.Groups[1].Value | ConvertFrom-Json
    $checksum = '0000000000000000'
    if ($Extension -eq '.anm2') {
        $bodyStart = $metadataMatch.Index + $metadataMatch.Length
        if ($Content.Substring($bodyStart).StartsWith("`r`n")) {
            $bodyStart += 2
        }
        elseif ($Content.Substring($bodyStart).StartsWith("`n")) {
            $bodyStart++
        }
        else {
            throw 'PTKメタデータ直後の改行を読み取れません。'
        }
        $checksum = [PsdToolKitDefinitionCyrb64]::ComputeHex(
            $Content.Substring($bodyStart)
        )
    }

    $metadata.checksum = $checksum
    $metadataJson = $metadata | ConvertTo-Json -Depth 100 -Compress
    $replacement = "--[==[PTK:$metadataJson]==]"
    $updatedContent =
        $Content.Substring(0, $metadataMatch.Index) +
        $replacement +
        $Content.Substring($metadataMatch.Index + $metadataMatch.Length)

    return [pscustomobject]@{
        Content = $updatedContent
        Checksum = $checksum
    }
}
