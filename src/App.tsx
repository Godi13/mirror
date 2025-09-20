import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface VersionInfo {
  current: string;
  latest: string;
  has_update: boolean;
  download_url?: string;
}

interface AppVersion {
  version: string;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [updateInfo, setUpdateInfo] = useState<VersionInfo | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [updateError, setUpdateError] = useState<string | null>(null);

  useEffect(() => {
    loadAppVersion();
  }, []);

  async function loadAppVersion() {
    try {
      const version: AppVersion = await invoke("get_app_version");
      setCurrentVersion(version.version);
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  }

  async function checkForUpdates() {
    setIsCheckingUpdate(true);
    setUpdateError(null);
    setUpdateInfo(null);
    try {
      const updateInfo: VersionInfo = await invoke("check_for_updates");
      setUpdateInfo(updateInfo);
    } catch (error) {
      console.error("Failed to check for updates:", error);
      setUpdateError(
        typeof error === "string" ? error : "检查更新失败，请稍后再试"
      );
    } finally {
      setIsCheckingUpdate(false);
    }
  }

  async function triggerAutoUpdate() {
    console.log("triggerAutoUpdate 被调用");
    try {
      console.log("开始调用 trigger_update_check");
      const result: string = await invoke("trigger_update_check");
      console.log("收到结果:", result);
      alert(`自动更新结果: ${result}`);
    } catch (error) {
      console.error("Failed to trigger auto update:", error);
      alert(`自动更新失败: ${JSON.stringify(error)}`);
    }
  }

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      {/* 版本信息区域 */}
      <div className="version-section">
        <p>当前版本: {currentVersion || "加载中..."}</p>
        <div style={{ display: "flex", gap: "10px", marginTop: "10px" }}>
          <button onClick={checkForUpdates} disabled={isCheckingUpdate}>
            {isCheckingUpdate ? "检查中..." : "检查更新"}
          </button>
          <button
            onClick={triggerAutoUpdate}
            style={{ backgroundColor: "#28a745" }}
          >
            自动更新 (Tauri)
          </button>
        </div>

        {updateError && (
          <div
            style={{
              marginTop: "10px",
              padding: "10px",
              border: "1px solid #dc3545",
              borderRadius: "5px",
              backgroundColor: "#f8d7da",
              color: "#721c24",
            }}
          >
            <p>❌ {updateError}</p>
          </div>
        )}
        {updateInfo && (
          <div
            style={{
              marginTop: "10px",
              padding: "10px",
              border: "1px solid #ccc",
              borderRadius: "5px",
            }}
          >
            {updateInfo.has_update ? (
              <div style={{ color: "green" }}>
                <p>🎉 发现新版本！</p>
                <p>当前版本: {updateInfo.current}</p>
                <p>最新版本: {updateInfo.latest}</p>
                {updateInfo.download_url && (
                  <a
                    href={updateInfo.download_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{ color: "blue", textDecoration: "underline" }}
                  >
                    下载更新
                  </a>
                )}
              </div>
            ) : (
              <div style={{ color: "gray" }}>
                <p>✅ 已是最新版本</p>
                <p>当前版本: {updateInfo.current}</p>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
