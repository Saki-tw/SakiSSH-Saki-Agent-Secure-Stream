// =============================================================================
// SakiSshDaemon.Interop — Services/ISessionRenewalHandler.cs
// SASS (Saki Agent Secure Stream) — Session Renewal Service Handler 介面
//
// 對應 sakissh.proto: rpc RenewSession(RenewSessionRequest) returns (RenewSessionResponse)
// 此介面供 C# Plugin 層實作 Session 續期的業務邏輯。
// 實際 gRPC 服務端由 Rust core 處理，C# 端透過此介面接收處理委派。
//
// 參考: draft-sakistudio-sass-00 §5.2
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System.Threading;
using System.Threading.Tasks;
using SakiSshDaemon.Interop.Messages;

namespace SakiSshDaemon.Interop.Services
{
    /// <summary>
    /// Session Renewal RPC handler 介面。
    /// <para>
    /// 對應 proto: <c>rpc RenewSession(RenewSessionRequest) returns (RenewSessionResponse)</c>
    /// </para>
    /// <para>
    /// Unary 模式：接收 <see cref="RenewSessionRequest"/>，回傳 <see cref="RenewSessionResponse"/>。
    /// Daemon 收到請求後，驗證 session 有效性，決定是否授予續期及時長。
    /// </para>
    /// </summary>
    public interface ISessionRenewalHandler
    {
        /// <summary>
        /// 處理 Session 續期請求。
        /// </summary>
        /// <param name="request">續期請求</param>
        /// <param name="cancellationToken">取消令牌</param>
        /// <returns>續期結果</returns>
        Task<RenewSessionResponse> HandleAsync(
            RenewSessionRequest request,
            CancellationToken cancellationToken = default);
    }
}
