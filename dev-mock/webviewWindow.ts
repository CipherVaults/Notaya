export const getCurrentWebviewWindow = () => ({ label: new URLSearchParams(location.search).get("label") ?? "main" });
