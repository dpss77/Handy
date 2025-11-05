import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { useSettings } from "@/hooks/useSettings";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import { AlertCircle, CheckCircle2, Loader2 } from "lucide-react";

export function OllamaSettings() {
  const { settings, updateSettings } = useSettings();
  const [isCheckingOllama, setIsCheckingOllama] = useState(false);
  const [ollamaAvailable, setOllamaAvailable] = useState<boolean | null>(null);
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = useState(false);
  const [testText, setTestText] = useState("hello world this is a test");
  const [testResult, setTestResult] = useState("");
  const [isTesting, setIsTesting] = useState(false);

  const enabled = settings?.enable_ollama ?? false;
  const url = settings?.ollama_url ?? "http://localhost:11434";
  const model = settings?.ollama_model ?? "llama3.2";
  const mode = settings?.ollama_mode ?? "disabled";
  const customPrompt = settings?.ollama_custom_prompt ?? "";

  useEffect(() => {
    if (enabled) {
      checkOllamaAvailability();
    }
  }, [enabled, url]);

  const checkOllamaAvailability = async () => {
    setIsCheckingOllama(true);
    try {
      const available = await invoke<boolean>("check_ollama_available", { url });
      setOllamaAvailable(available);

      if (available) {
        loadAvailableModels();
      }
    } catch (error) {
      console.error("Failed to check Ollama availability:", error);
      setOllamaAvailable(false);
    } finally {
      setIsCheckingOllama(false);
    }
  };

  const loadAvailableModels = async () => {
    setIsLoadingModels(true);
    try {
      const models = await invoke<string[]>("list_ollama_models", { url });
      setAvailableModels(models);
    } catch (error) {
      console.error("Failed to load Ollama models:", error);
      setAvailableModels([]);
    } finally {
      setIsLoadingModels(false);
    }
  };

  const testProcessing = async () => {
    setIsTesting(true);
    setTestResult("");
    try {
      const result = await invoke<string>("test_ollama_processing", {
        text: testText,
        model,
        url,
      });
      setTestResult(result);
    } catch (error) {
      console.error("Failed to test Ollama processing:", error);
      setTestResult(`Error: ${error}`);
    } finally {
      setIsTesting(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Ollama Integration</CardTitle>
        <CardDescription>
          Enhance transcriptions with local LLM post-processing (requires Ollama)
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Enable Toggle */}
        <div className="flex items-center justify-between">
          <div className="space-y-0.5">
            <Label htmlFor="enable-ollama">Enable Ollama</Label>
            <div className="text-sm text-muted-foreground">
              Use local LLM for post-processing transcriptions
            </div>
          </div>
          <Switch
            id="enable-ollama"
            checked={enabled}
            onCheckedChange={(checked) => {
              updateSettings({ enable_ollama: checked });
            }}
          />
        </div>

        {enabled && (
          <>
            {/* Ollama URL */}
            <div className="space-y-2">
              <Label htmlFor="ollama-url">Ollama Server URL</Label>
              <div className="flex gap-2">
                <Input
                  id="ollama-url"
                  value={url}
                  onChange={(e) => updateSettings({ ollama_url: e.target.value })}
                  placeholder="http://localhost:11434"
                  className="flex-1"
                />
                <Button
                  variant="outline"
                  onClick={checkOllamaAvailability}
                  disabled={isCheckingOllama}
                >
                  {isCheckingOllama ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    "Check"
                  )}
                </Button>
              </div>
              {ollamaAvailable !== null && (
                <div className="flex items-center gap-2 text-sm">
                  {ollamaAvailable ? (
                    <>
                      <CheckCircle2 className="h-4 w-4 text-green-500" />
                      <span className="text-green-500">Ollama is available</span>
                    </>
                  ) : (
                    <>
                      <AlertCircle className="h-4 w-4 text-red-500" />
                      <span className="text-red-500">
                        Ollama is not available. Make sure it's running.
                      </span>
                    </>
                  )}
                </div>
              )}
            </div>

            {/* Model Selection */}
            <div className="space-y-2">
              <Label htmlFor="ollama-model">Model</Label>
              <div className="flex gap-2">
                {isLoadingModels ? (
                  <div className="flex items-center gap-2 flex-1">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span className="text-sm text-muted-foreground">Loading models...</span>
                  </div>
                ) : availableModels.length > 0 ? (
                  <Select
                    value={model}
                    onValueChange={(value) => updateSettings({ ollama_model: value })}
                  >
                    <SelectTrigger className="flex-1">
                      <SelectValue placeholder="Select a model" />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels.map((m) => (
                        <SelectItem key={m} value={m}>
                          {m}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                ) : (
                  <Input
                    id="ollama-model"
                    value={model}
                    onChange={(e) => updateSettings({ ollama_model: e.target.value })}
                    placeholder="llama3.2"
                    className="flex-1"
                  />
                )}
                {ollamaAvailable && (
                  <Button
                    variant="outline"
                    onClick={loadAvailableModels}
                    disabled={isLoadingModels}
                  >
                    Refresh
                  </Button>
                )}
              </div>
              <div className="text-xs text-muted-foreground">
                Install models with: <code className="bg-muted px-1 rounded">ollama pull llama3.2</code>
              </div>
            </div>

            {/* Processing Mode */}
            <div className="space-y-2">
              <Label htmlFor="ollama-mode">Processing Mode</Label>
              <Select
                value={mode}
                onValueChange={(value) => updateSettings({ ollama_mode: value })}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select processing mode" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="disabled">Disabled</SelectItem>
                  <SelectItem value="punctuation">
                    Punctuation & Capitalization
                  </SelectItem>
                  <SelectItem value="summarize">Summarize</SelectItem>
                  <SelectItem value="commands">Extract Commands</SelectItem>
                  <SelectItem value="custom">Custom Prompt</SelectItem>
                </SelectContent>
              </Select>
              <div className="text-xs text-muted-foreground">
                {mode === "punctuation" && "Add proper punctuation and capitalization"}
                {mode === "summarize" && "Create a concise summary"}
                {mode === "commands" && "Extract action items and commands"}
                {mode === "custom" && "Use your own custom prompt"}
              </div>
            </div>

            {/* Custom Prompt */}
            {mode === "custom" && (
              <div className="space-y-2">
                <Label htmlFor="ollama-custom-prompt">Custom Prompt</Label>
                <Textarea
                  id="ollama-custom-prompt"
                  value={customPrompt}
                  onChange={(e) =>
                    updateSettings({ ollama_custom_prompt: e.target.value })
                  }
                  placeholder="Enter your custom system prompt..."
                  rows={4}
                  className="font-mono text-sm"
                />
                <div className="text-xs text-muted-foreground">
                  This prompt will be used as the system message for processing.
                </div>
              </div>
            )}

            {/* Test Section */}
            {ollamaAvailable && mode !== "disabled" && (
              <div className="space-y-2 border-t pt-4">
                <Label>Test Processing</Label>
                <Input
                  value={testText}
                  onChange={(e) => setTestText(e.target.value)}
                  placeholder="Enter test text..."
                />
                <Button
                  onClick={testProcessing}
                  disabled={isTesting || !testText}
                  className="w-full"
                >
                  {isTesting ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Processing...
                    </>
                  ) : (
                    "Test Processing"
                  )}
                </Button>
                {testResult && (
                  <div className="mt-2 p-3 bg-muted rounded-md">
                    <div className="text-sm font-medium mb-1">Result:</div>
                    <div className="text-sm">{testResult}</div>
                  </div>
                )}
              </div>
            )}

            {/* Info */}
            <div className="text-xs text-muted-foreground bg-muted p-3 rounded-md">
              <strong>Privacy Note:</strong> All processing happens locally on your machine.
              No data is sent to external servers.
              <br />
              <br />
              <strong>No Model Duplication:</strong> Ollama uses its own models (GGUF format)
              separate from Whisper/Parakeet (GGML format). Install Ollama from{" "}
              <a
                href="https://ollama.ai"
                className="underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                ollama.ai
              </a>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
