// =============================================================================
// SakiSshDaemon.Interop — Messages/RawFileTransferMessages.cs
// SASS (Saki Agent Secure Stream) — Raw File Transfer Message 定義
//
// 對應 sakissh.proto §7.3 Raw File Transfer 的 3 個 message：
//   - RawFileChunk (client-streaming payload)
//   - RawFileMetadata (首個 chunk 的 metadata)
//   - RawFileTransferResponse (server 回應)
//
// 注意：本專案不使用 protoc 自動產出，而是手動定義 POCO class。
// 欄位名稱與編號對應 proto 定義，序列化由 Rust core 處理。
//
// 參考: draft-sakistudio-sass-00 §7.3
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;

namespace SakiSshDaemon.Interop.Messages
{
    /// <summary>
    /// Raw File Transfer 的串流 chunk。
    /// <para>
    /// 對應 proto: <c>message RawFileChunk</c>
    /// 使用 oneof payload 語義：首個 chunk 必須為 Metadata，後續為 Data。
    /// </para>
    /// </summary>
    public sealed class RawFileChunk
    {
        /// <summary>
        /// 檔案 metadata（首個 chunk 必須設定此欄位）。
        /// 對應 proto field: metadata = 1
        /// </summary>
        public RawFileMetadata? Metadata { get; set; }

        /// <summary>
        /// 檔案資料片段。
        /// 對應 proto field: data = 2
        /// </summary>
        public byte[]? Data { get; set; }

        /// <summary>
        /// 判斷此 chunk 是否為 metadata chunk。
        /// 模擬 proto oneof 語義。
        /// </summary>
        public bool IsMetadata => Metadata != null;

        /// <summary>
        /// 判斷此 chunk 是否為 data chunk。
        /// 模擬 proto oneof 語義。
        /// </summary>
        public bool IsData => Data != null;

        /// <summary>
        /// 建立 metadata chunk。
        /// </summary>
        public static RawFileChunk FromMetadata(RawFileMetadata metadata)
        {
            if (metadata == null) throw new ArgumentNullException(nameof(metadata));
            return new RawFileChunk { Metadata = metadata };
        }

        /// <summary>
        /// 建立 data chunk。
        /// </summary>
        public static RawFileChunk FromData(byte[] data)
        {
            if (data == null) throw new ArgumentNullException(nameof(data));
            return new RawFileChunk { Data = data };
        }
    }

    /// <summary>
    /// Raw File Transfer 的檔案 metadata。
    /// <para>
    /// 對應 proto: <c>message RawFileMetadata</c>
    /// 包含 remote_path、total_size、offset（斷點續傳）、checksum_sha256（完整性驗證）。
    /// </para>
    /// </summary>
    public sealed class RawFileMetadata
    {
        /// <summary>
        /// 遠端檔案路徑。
        /// 對應 proto field: remote_path = 1
        /// </summary>
        public string RemotePath { get; set; } = string.Empty;

        /// <summary>
        /// 檔案總大小（bytes）。
        /// 對應 proto field: total_size = 2
        /// </summary>
        public ulong TotalSize { get; set; }

        /// <summary>
        /// 斷點續傳：從此 offset 開始寫入。
        /// 對應 proto field: offset = 3
        /// </summary>
        public ulong Offset { get; set; }

        /// <summary>
        /// 完整性驗證（SHA-256 hex string）。
        /// 對應 proto field: checksum_sha256 = 4
        /// </summary>
        public string ChecksumSha256 { get; set; } = string.Empty;
    }

    /// <summary>
    /// Raw File Transfer 的伺服器回應。
    /// <para>
    /// 對應 proto: <c>message RawFileTransferResponse</c>
    /// </para>
    /// </summary>
    public sealed class RawFileTransferResponse
    {
        /// <summary>
        /// 傳輸是否成功。
        /// 對應 proto field: success = 1
        /// </summary>
        public bool Success { get; set; }

        /// <summary>
        /// 回應訊息（成功/失敗原因）。
        /// 對應 proto field: message = 2
        /// </summary>
        public string Message { get; set; } = string.Empty;

        /// <summary>
        /// 實際寫入的位元組數。
        /// 對應 proto field: bytes_written = 3
        /// </summary>
        public ulong BytesWritten { get; set; }

        /// <summary>
        /// 寫入後校驗（SHA-256 hex string）。
        /// 對應 proto field: checksum_sha256 = 4
        /// </summary>
        public string ChecksumSha256 { get; set; } = string.Empty;
    }
}
