// =============================================================================
// SakiSshDaemon.Interop — Services/IRawFileTransferHandler.cs
// SASS (Saki Agent Secure Stream) — Raw File Transfer Service Handler 介面
//
// 對應 sakissh.proto: rpc RawFileTransfer(stream RawFileChunk) returns (RawFileTransferResponse)
// 此介面供 C# Plugin 層實作 Raw File Transfer 的業務邏輯。
// 實際 gRPC 服務端由 Rust core 處理，C# 端透過此介面接收處理委派。
//
// 參考: draft-sakistudio-sass-05 §7.3
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using SakiSshDaemon.Interop.Messages;

namespace SakiSshDaemon.Interop.Services
{
    /// <summary>
    /// Raw File Transfer RPC handler 介面。
    /// <para>
    /// 對應 proto: <c>rpc RawFileTransfer(stream RawFileChunk) returns (RawFileTransferResponse)</c>
    /// </para>
    /// <para>
    /// Client streaming 模式：接收一連串 <see cref="RawFileChunk"/>，回傳 <see cref="RawFileTransferResponse"/>。
    /// 首個 chunk 必須包含 <see cref="RawFileMetadata"/>（路徑、大小、offset、checksum）。
    /// </para>
    /// </summary>
    public interface IRawFileTransferHandler
    {
        /// <summary>
        /// 處理 Raw File Transfer 串流。
        /// </summary>
        /// <param name="chunks">Client 傳送的串流 chunk 序列</param>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>傳輸結果</returns>
        Task<RawFileTransferResponse> HandleAsync(
            IAsyncEnumerable<RawFileChunk> chunks,
            CancellationToken cancellationToken = default);
    }
}
