import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ParsedDocument {
  content: string;
  paragraphs: Array<{
    index: number;
    text: string;
    style_name: string | null;
    font_name: string | null;
    font_size: number | null;
    line_spacing: number | null;
  }>;
  metadata: {
    page_count: number;
    word_count: number;
    title: string | null;
    author: string | null;
  };
  format_info: {
    file_type: string;
    styles: Array<{
      name: string;
      font_name: string | null;
      font_size: number | null;
      is_bold: boolean;
      alignment: string | null;
    }>;
    fonts: string[];
    has_toc: boolean;
    has_bibliography: boolean;
  };
}

interface FormatIssue {
  issue_type: string;
  description: string;
  location: {
    page: number | null;
    paragraph: number | null;
    section: string | null;
  };
  severity: string;
  suggestion: string;
}

interface FormatCheckResult {
  issues: FormatIssue[];
  summary: {
    total_issues: number;
    critical: number;
    major: number;
    minor: number;
    overall_assessment: string;
  };
}

function App() {
  const [formatFile, setFormatFile] = useState<File | null>(null);
  const [paperFile, setPaperFile] = useState<File | null>(null);
  const [formatDoc, setFormatDoc] = useState<ParsedDocument | null>(null);
  const [paperDoc, setPaperDoc] = useState<ParsedDocument | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [apiKey, setApiKey] = useState("");
  const [apiProvider, setApiProvider] = useState("minimax");
  const [apiModel, setApiModel] = useState("abab6.5s-chat");
  const [isSavingSettings, setIsSavingSettings] = useState(false);
  const [settingsSaved, setSettingsSaved] = useState(false);
  const [checkResult, setCheckResult] = useState<FormatCheckResult | null>(null);

  // Save API settings
  const saveSettings = async () => {
    if (!apiKey.trim()) {
      setError("请输入 API Key");
      return;
    }

    setIsSavingSettings(true);
    setError(null);

    try {
      await invoke("set_llm_config", {
        provider: apiProvider,
        apiKey: apiKey,
        model: apiModel,
      });
      setSettingsSaved(true);
      setTimeout(() => {
        setSettingsSaved(false);
        setShowSettings(false);
      }, 1500);
    } catch (err) {
      setError(`保存设置失败: ${err}`);
    } finally {
      setIsSavingSettings(false);
    }
  };

  const checkFormat = async () => {
    if (!formatDoc || !paperDoc) {
      setError("请上传格式要求和论文文件");
      return;
    }

    setIsChecking(true);
    setError(null);
    setCheckResult(null);

    try {
      // Call backend to check format with LLM
      // Pass the format requirements content
      const result = await invoke<FormatCheckResult>("check_format_with_document", {
        formatRequirements: formatDoc.content,
        parsedDocument: paperDoc,
      });

      setCheckResult(result);
    } catch (err) {
      // Fallback to simple local check if LLM fails
      console.error("LLM check failed:", err);

      // Simple local check as fallback
      const foundIssues: FormatIssue[] = [];
      const formatText = formatDoc.content.toLowerCase();
      const paperText = paperDoc.content;

      if (formatText.includes("1.5") && !paperText.includes("1.5")) {
        foundIssues.push({
          issue_type: "行距",
          description: "论文可能未使用1.5倍行距",
          location: { page: null, paragraph: null, section: "正文" },
          severity: "major",
          suggestion: "请将正文行距设置为1.5倍",
        });
      }

      if (formatText.includes("宋体") && !paperText.includes("宋体")) {
        foundIssues.push({
          issue_type: "字体",
          description: "论文可能未使用宋体",
          location: { page: null, paragraph: null, section: "正文" },
          severity: "major",
          suggestion: "请将正文字体设置为宋体",
        });
      }

      if (formatText.includes("2.5") || formatText.includes("页边距")) {
        foundIssues.push({
          issue_type: "页边距",
          description: "请确保页边距符合要求（上下左右2.5cm）",
          location: { page: null, paragraph: null, section: "页面设置" },
          severity: "minor",
          suggestion: "检查页面设置中的页边距",
        });
      }

      if (formatText.includes("参考文献") && !paperText.includes("[1]")) {
        foundIssues.push({
          issue_type: "参考文献",
          description: "参考文献格式可能不符合GB/T 7714标准",
          location: { page: null, paragraph: null, section: "参考文献" },
          severity: "major",
          suggestion: "请按GB/T 7714-2015格式排版参考文献",
        });
      }

      setCheckResult({
        issues: foundIssues,
        summary: {
          total_issues: foundIssues.length,
          critical: foundIssues.filter((i) => i.severity === "critical").length,
          major: foundIssues.filter((i) => i.severity === "major").length,
          minor: foundIssues.filter((i) => i.severity === "minor").length,
          overall_assessment: foundIssues.length === 0 ? "格式基本符合要求" : "存在格式问题需要修改",
        },
      });
    } finally {
      setIsChecking(false);
    }
  };

  const exportReport = () => {
    if (!checkResult) return;

    const report = `
论文格式检查报告
================

检查日期: ${new Date().toLocaleString()}

文件: ${paperFile?.name || "未知"}
格式要求: ${formatFile?.name || "未上传"}

总体评估: ${checkResult.summary.overall_assessment}

发现问题:
${checkResult.issues.length === 0 ? "未发现明显格式问题" : checkResult.issues.map((issue, i) => `
${i + 1}. [${issue.severity.toUpperCase()}] ${issue.issue_type}
   问题: ${issue.description}
   建议: ${issue.suggestion}
   位置: ${issue.location.section || "未知"}
`).join("")}

总计: ${checkResult.summary.total_issues} 个问题
- 严重: ${checkResult.summary.critical}
- 重要: ${checkResult.summary.major}
- 轻微: ${checkResult.summary.minor}

---
Generated by PaperFormatChecker
`;

    const blob = new Blob([report], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `论文格式检查报告_${Date.now()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 py-4 flex justify-between items-center">
          <div>
            <h1 className="text-2xl font-bold text-gray-900">论文格式检查器</h1>
            <p className="text-sm text-gray-500 mt-1">
              上传论文格式要求和您的论文，AI帮您检查格式问题
            </p>
          </div>
          <button
            onClick={() => setShowSettings(true)}
            className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors text-sm flex items-center gap-2"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.953 1.57c1.876.877 3.89 1.34 5.93 1.24M15 10.5a3 3 0 11-6 0 3 3 0 016 0zm6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            API 设置
          </button>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 py-6">
        {/* Error Message */}
        {error && (
          <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg">
            <p className="text-red-800">{error}</p>
          </div>
        )}

        {/* Upload Section */}
        <div className="grid md:grid-cols-2 gap-6 mb-6">
          {/* Format Requirements */}
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-semibold mb-4">1. 上传格式要求</h2>
            <p className="text-sm text-gray-500 mb-4">
              上传学校的论文撰写格式要求（Word、PDF或TXT）
            </p>
            <div className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center hover:border-primary-500 transition-colors">
              <input
                type="file"
                accept=".docx,.pdf,.txt"
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (file) {
                    try {
                      // Read file content as text for now
                      const text = await file.text();
                      setFormatFile(file);
                      setFormatDoc({
                        content: text,
                        paragraphs: text.split("\n").map((line, i) => ({
                          index: i,
                          text: line,
                          style_name: null,
                          font_name: null,
                          font_size: null,
                          line_spacing: null,
                        })),
                        metadata: {
                          page_count: 1,
                          word_count: text.split(/\s+/).length,
                          title: null,
                          author: null,
                        },
                        format_info: {
                          file_type: "txt",
                          styles: [],
                          fonts: [],
                          has_toc: false,
                          has_bibliography: false,
                        },
                      });
                    } catch (err) {
                      setError(`读取文件失败: ${err}`);
                    }
                  }
                }}
                className="hidden"
                id="format-file"
              />
              <label htmlFor="format-file" className="cursor-pointer">
                <div className="text-gray-600">
                  {formatFile ? (
                    <div>
                      <p className="font-medium text-green-600">已选择: {formatFile.name}</p>
                      <p className="text-sm mt-1">点击重新选择</p>
                    </div>
                  ) : (
                    <div>
                      <p className="text-4xl mb-2">📄</p>
                      <p>点击或拖拽文件到这里</p>
                      <p className="text-xs text-gray-400 mt-1">支持 .docx, .pdf, .txt</p>
                    </div>
                  )}
                </div>
              </label>
            </div>
          </div>

          {/* Paper Document */}
          <div className="bg-white rounded-lg shadow p-6">
            <h2 className="text-lg font-semibold mb-4">2. 上传论文</h2>
            <p className="text-sm text-gray-500 mb-4">
              上传您已完成待检查的论文（Word或PDF）
            </p>
            <div className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center hover:border-primary-500 transition-colors">
              <input
                type="file"
                accept=".docx,.pdf,.txt"
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (file) {
                    try {
                      const text = await file.text();
                      setPaperFile(file);
                      setPaperDoc({
                        content: text,
                        paragraphs: text.split("\n").map((line, i) => ({
                          index: i,
                          text: line,
                          style_name: null,
                          font_name: null,
                          font_size: null,
                          line_spacing: null,
                        })),
                        metadata: {
                          page_count: Math.ceil(text.length / 2000),
                          word_count: text.split(/\s+/).length,
                          title: null,
                          author: null,
                        },
                        format_info: {
                          file_type: "txt",
                          styles: [],
                          fonts: [],
                          has_toc: false,
                          has_bibliography: false,
                        },
                      });
                    } catch (err) {
                      setError(`读取文件失败: ${err}`);
                    }
                  }
                }}
                className="hidden"
                id="paper-file"
              />
              <label htmlFor="paper-file" className="cursor-pointer">
                <div className="text-gray-600">
                  {paperFile ? (
                    <div>
                      <p className="font-medium text-green-600">已选择: {paperFile.name}</p>
                      <p className="text-sm mt-1">点击重新选择</p>
                    </div>
                  ) : (
                    <div>
                      <p className="text-4xl mb-2">📝</p>
                      <p>点击或拖拽文件到这里</p>
                      <p className="text-xs text-gray-400 mt-1">支持 .docx, .pdf, .txt</p>
                    </div>
                  )}
                </div>
              </label>
            </div>
          </div>
        </div>

        {/* Check Button */}
        <div className="text-center mb-6">
          <button
            onClick={checkFormat}
            disabled={!formatDoc || !paperDoc || isChecking}
            className={`px-8 py-3 rounded-lg font-medium text-white transition-colors ${
              formatDoc && paperDoc && !isChecking
                ? "bg-primary-600 hover:bg-primary-700"
                : "bg-gray-400 cursor-not-allowed"
            }`}
          >
            {isChecking ? (
              <span className="flex items-center gap-2">
                <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                检查中...
              </span>
            ) : (
              "开始检查"
            )}
          </button>
        </div>

        {/* Results Section */}
        {checkResult && (
          <div className="bg-white rounded-lg shadow p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">检查结果</h2>
              <button
                onClick={exportReport}
                className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
              >
                导出报告
              </button>
            </div>

            {/* Summary */}
            <div className="grid grid-cols-4 gap-4 mb-6">
              <div className="bg-gray-50 rounded-lg p-4 text-center">
                <p className="text-2xl font-bold">{checkResult.summary.total_issues}</p>
                <p className="text-sm text-gray-500">总问题数</p>
              </div>
              <div className="bg-red-50 rounded-lg p-4 text-center">
                <p className="text-2xl font-bold text-red-600">
                  {checkResult.summary.critical}
                </p>
                <p className="text-sm text-red-500">严重</p>
              </div>
              <div className="bg-orange-50 rounded-lg p-4 text-center">
                <p className="text-2xl font-bold text-orange-600">
                  {checkResult.summary.major}
                </p>
                <p className="text-sm text-orange-500">重要</p>
              </div>
              <div className="bg-yellow-50 rounded-lg p-4 text-center">
                <p className="text-2xl font-bold text-yellow-600">
                  {checkResult.summary.minor}
                </p>
                <p className="text-sm text-yellow-500">轻微</p>
              </div>
            </div>

            {/* Overall Assessment */}
            <div className="bg-blue-50 rounded-lg p-4 mb-6">
              <p className="text-blue-800">{checkResult.summary.overall_assessment}</p>
            </div>

            {/* Issue List */}
            {checkResult.issues.length > 0 ? (
              <div className="space-y-3">
                {checkResult.issues.map((issue, index) => (
                  <div
                    key={index}
                    className={`p-4 rounded-lg border-l-4 ${
                      issue.severity === "critical"
                        ? "bg-red-50 border-red-500"
                        : issue.severity === "major"
                        ? "bg-orange-50 border-orange-500"
                        : "bg-yellow-50 border-yellow-500"
                    }`}
                  >
                    <div className="flex justify-between items-start">
                      <div>
                        <span
                          className={`inline-block px-2 py-1 rounded text-xs font-medium ${
                            issue.severity === "critical"
                              ? "bg-red-100 text-red-800"
                              : issue.severity === "major"
                              ? "bg-orange-100 text-orange-800"
                              : "bg-yellow-100 text-yellow-800"
                          }`}
                        >
                          {issue.severity === "critical"
                            ? "严重"
                            : issue.severity === "major"
                            ? "重要"
                            : "轻微"}
                        </span>
                        <span className="ml-2 font-medium">{issue.issue_type}</span>
                      </div>
                      {issue.location.section && (
                        <span className="text-sm text-gray-500">
                          {issue.location.section}
                        </span>
                      )}
                    </div>
                    <p className="mt-2 text-gray-700">{issue.description}</p>
                    {issue.suggestion && (
                      <p className="mt-1 text-sm text-blue-600">💡 建议: {issue.suggestion}</p>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8">
                <div className="text-6xl mb-4">✅</div>
                <h3 className="text-lg font-semibold text-gray-900">检查完成</h3>
                <p className="text-gray-500 mt-2">未发现明显的格式问题</p>
              </div>
            )}
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="bg-white border-t mt-auto">
        <div className="max-w-7xl mx-auto px-4 py-4 text-center text-sm text-gray-500">
          PaperFormatChecker v1.0.0
        </div>
      </footer>

      {/* Settings Modal */}
      {showSettings && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-bold">API 设置</h2>
              <button
                onClick={() => setShowSettings(false)}
                className="text-gray-500 hover:text-gray-700"
              >
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  API Provider
                </label>
                <select
                  value={apiProvider}
                  onChange={(e) => setApiProvider(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                >
                  <option value="minimax">MiniMax (推荐)</option>
                  <option value="openai">OpenAI</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  API Key
                </label>
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="输入您的 API Key"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                />
                <p className="text-xs text-gray-500 mt-1">
                  {apiProvider === "minimax"
                    ? "获取 API Key: https://platform.minimax.io/"
                    : "获取 API Key: https://platform.openai.com/"}
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Model
                </label>
                <select
                  value={apiModel}
                  onChange={(e) => setApiModel(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                >
                  {apiProvider === "minimax" ? (
                    <>
                      <option value="abab6.5s-chat">abab6.5s-chat (推荐)</option>
                      <option value="abab6.5-chat">abab6.5-chat</option>
                    </>
                  ) : (
                    <>
                      <option value="gpt-4o-mini">GPT-4o Mini (推荐)</option>
                      <option value="gpt-4o">GPT-4o</option>
                      <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                    </>
                  )}
                </select>
              </div>

              {error && (
                <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
                  <p className="text-sm text-red-800">{error}</p>
                </div>
              )}

              <button
                onClick={saveSettings}
                disabled={isSavingSettings}
                className="w-full py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:bg-gray-400"
              >
                {isSavingSettings ? "保存中..." : settingsSaved ? "✓ 已保存" : "保存设置"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;