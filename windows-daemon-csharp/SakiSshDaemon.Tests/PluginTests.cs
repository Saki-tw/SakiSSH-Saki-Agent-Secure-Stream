// =============================================================================
// SakiSshDaemon.Tests — PluginTests.cs
// SASS Windows Plugins 單元測試
//
// 使用 xUnit 測試所有 7 個 Plugins 的基本功能。
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

using System;
using System.Collections.Generic;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using SakiSshDaemon.Plugins;
using Xunit;

namespace SakiSshDaemon.Tests
{
    /// <summary>
    /// Plugin #1: ChaCha20-Poly1305 認知挑戰測試
    /// </summary>
    public class ChaCha20ChallengeTests
    {
        private readonly ChaCha20Challenge _plugin;

        public ChaCha20ChallengeTests()
        {
            _plugin = new ChaCha20Challenge(
                NullLogger<ChaCha20Challenge>.Instance);
        }

        [Fact]
        public async Task Initialize_ReturnsTrue()
        {
            bool result = await _plugin.InitializeAsync();
            Assert.True(result);
            Assert.True(_plugin.IsHealthy);
        }

        [Fact]
        public async Task GenerateChallenge_ReturnsNonceAndCiphertext()
        {
            await _plugin.InitializeAsync();
            var (nonce, ciphertext) = _plugin.GenerateChallenge();

            // Nonce 必須為 12 bytes（RFC 8439）
            Assert.Equal(12, nonce.Length);

            // Ciphertext = 64 bytes plaintext + 16 bytes tag = 80 bytes
            Assert.Equal(80, ciphertext.Length);
        }

        [Fact]
        public async Task GenerateChallenge_EachCallProducesUniqueNonce()
        {
            await _plugin.InitializeAsync();
            var (nonce1, _) = _plugin.GenerateChallenge();
            var (nonce2, _) = _plugin.GenerateChallenge();

            Assert.NotEqual(nonce1, nonce2);
        }

        [Fact]
        public async Task TryVerifyAny_InvalidResponse_ReturnsFalse()
        {
            await _plugin.InitializeAsync();
            _plugin.GenerateChallenge();

            // 錯誤的回應應該驗證失敗
            byte[] wrongResponse = new byte[64];
            Array.Fill(wrongResponse, (byte)0xFF);

            bool result = _plugin.TryVerifyAny(wrongResponse);
            // 因為 wrongResponse 不是正確的明文，應該失敗
            // （除非碰巧產生了全 0xFF 的明文，機率為 2^-512）
            Assert.False(result);
        }

        [Fact]
        public void Name_IsCorrect()
        {
            Assert.Equal("ChaCha20-Poly1305 Cognitive Challenge", _plugin.Name);
        }

        [Fact]
        public void RfcSection_IsCorrect()
        {
            Assert.Equal("Appendix C.1 (chacha20-challenge)", _plugin.RfcSection);
        }
    }

    /// <summary>
    /// Plugin #2: TLS Exporter Binding 測試
    /// </summary>
    public class TlsExporterBindingTests
    {
        private readonly TlsExporterBinding _plugin;

        public TlsExporterBindingTests()
        {
            _plugin = new TlsExporterBinding(
                NullLogger<TlsExporterBinding>.Instance);
        }

        [Fact]
        public void DeriveEkm_Returns44Bytes()
        {
            byte[] sessionUuid = new byte[16];
            new Random(42).NextBytes(sessionUuid);

            var ekm = _plugin.DeriveEkm(sessionUuid);

            // EKM 必須為 44 bytes（32 key + 12 nonce）
            Assert.Equal(44, ekm.Raw.Length);
            Assert.Equal(32, ekm.ChaChaKey.Length);
            Assert.Equal(12, ekm.ChaChaNonce.Length);
        }

        [Fact]
        public void DeriveEkm_SameInput_ProducesSameOutput()
        {
            byte[] sessionUuid = new byte[16];
            new Random(42).NextBytes(sessionUuid);

            var ekm1 = _plugin.DeriveEkm(sessionUuid);
            var ekm2 = _plugin.DeriveEkm(sessionUuid);

            Assert.Equal(ekm1.Raw, ekm2.Raw);
        }

        [Fact]
        public void DeriveEkm_InvalidUuidLength_ThrowsArgumentException()
        {
            byte[] shortUuid = new byte[8];
            Assert.Throws<ArgumentException>(() => _plugin.DeriveEkm(shortUuid));
        }

        [Fact]
        public void VerifyEkmHmac_ValidHmac_ReturnsTrue()
        {
            byte[] sessionUuid = new byte[16];
            new Random(42).NextBytes(sessionUuid);
            var ekm = _plugin.DeriveEkm(sessionUuid);

            byte[] plaintext = Encoding.UTF8.GetBytes("test-plaintext");

            // 計算正確的 HMAC
            byte[] correctHmac;
            using (var hmac = new System.Security.Cryptography.HMACSHA256(ekm.Raw))
            {
                correctHmac = hmac.ComputeHash(plaintext);
            }

            bool result = _plugin.VerifyEkmHmac(ekm, plaintext, correctHmac);
            Assert.True(result);
        }

        [Fact]
        public void VerifyEkmHmac_InvalidHmac_ReturnsFalse()
        {
            byte[] sessionUuid = new byte[16];
            new Random(42).NextBytes(sessionUuid);
            var ekm = _plugin.DeriveEkm(sessionUuid);

            byte[] plaintext = Encoding.UTF8.GetBytes("test-plaintext");
            byte[] wrongHmac = new byte[32]; // 全零的 HMAC

            bool result = _plugin.VerifyEkmHmac(ekm, plaintext, wrongHmac);
            Assert.False(result);
        }

        [Fact]
        public void ExporterLabel_MatchesRfc()
        {
            Assert.Equal("EXPORTER-sakissh-chacha20-v14",
                TlsExporterBinding.ExporterLabel);
        }

        [Fact]
        public void EkmLength_Is44()
        {
            Assert.Equal(44, TlsExporterBinding.EkmLength);
        }
    }

    /// <summary>
    /// Plugin #3: Tarpit Buffer 測試
    /// </summary>
    public class TarpitBufferTests
    {
        private readonly TarpitBuffer _plugin;

        public TarpitBufferTests()
        {
            _plugin = new TarpitBuffer(
                NullLogger<TarpitBuffer>.Instance);
        }

        [Fact]
        public async Task Initialize_ReturnsTrue()
        {
            bool result = await _plugin.InitializeAsync();
            Assert.True(result);
        }

        [Fact]
        public void TarpitConfig_DefaultValues_MatchRfc()
        {
            var config = TarpitConfig.Default;

            Assert.Equal(40 * 1024 * 1024, config.TotalBytes); // 40 MiB
            Assert.Equal(64 * 1024, config.ChunkSize);         // 64 KiB
            Assert.Equal(500, config.DelayMs);                  // 500ms
        }

        [Fact]
        public async Task Engulf_SmallConfig_CompletesSuccessfully()
        {
            await _plugin.InitializeAsync();

            int totalReceived = 0;
            var config = new TarpitConfig
            {
                TotalBytes = 128 * 1024,  // 128 KiB (2 chunks)
                ChunkSize = 64 * 1024,    // 64 KiB
                DelayMs = 10,             // 快速測試
            };

            await _plugin.EngulfAsync(
                writeCallback: (data, len) =>
                {
                    totalReceived += len;
                    return Task.CompletedTask;
                },
                config: config);

            Assert.Equal(config.TotalBytes, totalReceived);
        }

        [Fact]
        public void ActiveCount_InitiallyZero()
        {
            Assert.Equal(0, TarpitBuffer.ActiveCount);
        }
    }

    /// <summary>
    /// Plugin #4: AuditChain 測試
    /// </summary>
    public class AuditChainTests
    {
        private readonly AuditChain _plugin;

        public AuditChainTests()
        {
            _plugin = new AuditChain(
                NullLogger<AuditChain>.Instance);
        }

        [Fact]
        public async Task Initialize_ReturnsTrue()
        {
            bool result = await _plugin.InitializeAsync();
            Assert.True(result);
            _plugin.Dispose();
        }

        [Fact]
        public async Task Log_AuthSuccess_DoesNotThrow()
        {
            await _plugin.InitializeAsync();

            _plugin.Log(new AuthSuccessEvent
            {
                AgentName = "test-agent",
                SessionId = "test-session-123",
                PublicKeyHex = "deadbeef",
            });

            // 等待背景寫入完成
            await Task.Delay(200);
            _plugin.Dispose();
        }

        [Fact]
        public void Name_IsCorrect()
        {
            Assert.Equal("ED25519 Hash Chain Audit Log", _plugin.Name);
        }
    }

    /// <summary>
    /// Plugin #5: Vi Swap 測試
    /// </summary>
    public class ViSwapTests
    {
        private readonly ViSwap _plugin;

        public ViSwapTests()
        {
            _plugin = new ViSwap(
                NullLogger<ViSwap>.Instance);
        }

        [Fact]
        public void BuildViScreen_ContainsAllFiveAnsiEscapes()
        {
            string screen = _plugin.BuildViScreen();

            // RFC §C.5 定義的 5 個 ANSI escape
            Assert.Contains("\x1b[?1049h", screen); // #1 Enter alternate screen
            Assert.Contains("\x1b[2J", screen);      // #2 Clear screen
            Assert.Contains("\x1b[H", screen);       // #3 Cursor home
            Assert.Contains("\x1b[?25l", screen);    // #4 Hide cursor
            Assert.Contains("\x1b[24;1H", screen);   // #5 Bottom status line
        }

        [Fact]
        public void BuildViScreen_ContainsTildeLines()
        {
            string screen = _plugin.BuildViScreen();
            Assert.Contains("~\r\n", screen);
        }

        [Fact]
        public void BuildViScreen_ContainsSassDefenseMessage()
        {
            string screen = _plugin.BuildViScreen();
            Assert.Contains("SASS Active Defense: Vi-Swap Engaged", screen);
            Assert.Contains("13Policy Dangerous Command Violation", screen);
        }

        [Fact]
        public void BuildViScreen_ContainsStatusBar()
        {
            string screen = _plugin.BuildViScreen();
            Assert.Contains("SASS Vi-Swap Mode [Read-Only]", screen);
        }
    }

    /// <summary>
    /// Plugin #6: BranchManager 測試
    /// </summary>
    public class BranchManagerTests
    {
        private readonly BranchManager _plugin;

        public BranchManagerTests()
        {
            _plugin = new BranchManager(
                NullLogger<BranchManager>.Instance);
        }

        [Fact]
        public async Task Initialize_ReturnsTrue()
        {
            bool result = await _plugin.InitializeAsync();
            Assert.True(result);
        }

        [Fact]
        public void CreateMicroBranch_NonExistentDir_ReturnsNull()
        {
            string? result = _plugin.CreateMicroBranch(
                "test-session",
                "/nonexistent/directory/12345");
            Assert.Null(result);
        }

        [Fact]
        public void CreateMicroBranch_ExistingDir_ReturnsBranchPath()
        {
            // 使用臨時目錄作為 target
            string tempDir = System.IO.Path.Combine(
                System.IO.Path.GetTempPath(),
                "sass_test_branch_" + Guid.NewGuid().ToString("N")[..8]);
            System.IO.Directory.CreateDirectory(tempDir);

            try
            {
                string sessionId = "test-" + Guid.NewGuid().ToString("N")[..8];
                string? branchPath = _plugin.CreateMicroBranch(sessionId, tempDir);

                Assert.NotNull(branchPath);
                Assert.True(System.IO.Directory.Exists(branchPath));

                // 清理
                _plugin.DropBranch(sessionId);
            }
            finally
            {
                System.IO.Directory.Delete(tempDir, true);
            }
        }

        [Fact]
        public void Name_IsCorrect()
        {
            Assert.Equal("Transparent Branching (Branch Manager)", _plugin.Name);
        }
    }

    /// <summary>
    /// Plugin #7: EnvInjector 測試
    /// </summary>
    public class EnvInjectorTests
    {
        private readonly EnvInjector _plugin;

        public EnvInjectorTests()
        {
            _plugin = new EnvInjector(
                NullLogger<EnvInjector>.Instance);
        }

        [Fact]
        public async Task Initialize_ReturnsTrue()
        {
            bool result = await _plugin.InitializeAsync();
            Assert.True(result);
        }

        [Fact]
        public void InjectVolumeReductionEnv_NpmCommand_InjectsNpmAndYarnVars()
        {
            var env = new Dictionary<string, string>();
            var result = _plugin.InjectVolumeReductionEnv("npm install express", env);

            Assert.True(result.ContainsKey("npm_config_cache"));
            Assert.True(result.ContainsKey("YARN_CACHE_FOLDER"));
            Assert.Contains("sass_vol", result["npm_config_cache"]);
        }

        [Fact]
        public void InjectVolumeReductionEnv_CargoCommand_InjectsCargoVars()
        {
            var env = new Dictionary<string, string>();
            var result = _plugin.InjectVolumeReductionEnv("cargo build --release", env);

            Assert.True(result.ContainsKey("CARGO_TARGET_DIR"));
            Assert.True(result.ContainsKey("CARGO_HOME"));
        }

        [Fact]
        public void InjectVolumeReductionEnv_PipCommand_InjectsPipVar()
        {
            var env = new Dictionary<string, string>();
            var result = _plugin.InjectVolumeReductionEnv("pip install requests", env);

            Assert.True(result.ContainsKey("PIP_CACHE_DIR"));
        }

        [Fact]
        public void InjectVolumeReductionEnv_AnyCommand_AlwaysInjectsTmpdir()
        {
            var env = new Dictionary<string, string>();
            var result = _plugin.InjectVolumeReductionEnv("ls -la", env);

            Assert.True(result.ContainsKey("TMPDIR"));
            // Windows 專用
            Assert.True(result.ContainsKey("TEMP"));
            Assert.True(result.ContainsKey("TMP"));
        }

        [Fact]
        public void InjectVolumeReductionEnv_YarnCommand_MatchesNodeDetection()
        {
            var env = new Dictionary<string, string>();
            var result = _plugin.InjectVolumeReductionEnv("yarn add lodash", env);

            Assert.True(result.ContainsKey("npm_config_cache"));
            Assert.True(result.ContainsKey("YARN_CACHE_FOLDER"));
        }

        [Fact]
        public void Name_IsCorrect()
        {
            Assert.Equal("Volatile Cache Redirection (EnvInjector)", _plugin.Name);
        }

        [Fact]
        public void RfcSection_IsCorrect()
        {
            Assert.Equal("Appendix C.7 (volatile-cache)", _plugin.RfcSection);
        }
    }
}
