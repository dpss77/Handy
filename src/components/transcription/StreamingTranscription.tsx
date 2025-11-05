import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Loader2, Mic, CheckCircle2 } from "lucide-react";
import { PartialTranscription } from "@/lib/types";

interface StreamingTranscriptionProps {
  /** Whether streaming is currently active */
  isActive: boolean;
  /** Optional className for styling */
  className?: string;
}

/**
 * StreamingTranscription component displays real-time partial transcription results
 * as they arrive from the backend during recording.
 */
export function StreamingTranscription({
  isActive,
  className = "",
}: StreamingTranscriptionProps) {
  const [partialResults, setPartialResults] = useState<PartialTranscription[]>([]);
  const [isRecording, setIsRecording] = useState(false);

  useEffect(() => {
    if (!isActive) {
      // Clear results when not active
      setPartialResults([]);
      setIsRecording(false);
      return;
    }

    setIsRecording(true);

    // Listen for partial transcription events
    const unlistenPartial = listen<PartialTranscription>(
      "partial-transcription",
      (event) => {
        const partial = event.payload;

        setPartialResults((prev) => {
          // Replace existing result for this chunk or add new one
          const existingIndex = prev.findIndex(
            (p) => p.chunk_index === partial.chunk_index
          );

          if (existingIndex >= 0) {
            const updated = [...prev];
            updated[existingIndex] = partial;
            return updated;
          } else {
            return [...prev, partial].sort((a, b) => a.chunk_index - b.chunk_index);
          }
        });

        // If this is a final result, mark recording as complete
        if (partial.is_final) {
          setIsRecording(false);
        }
      }
    );

    return () => {
      unlistenPartial.then((fn) => fn());
    };
  }, [isActive]);

  if (!isActive && partialResults.length === 0) {
    return null;
  }

  const totalChunks = partialResults[0]?.total_chunks;
  const processedChunks = partialResults.filter((r) => r.text.trim().length > 0).length;

  return (
    <Card className={`fixed top-4 right-4 w-96 shadow-lg z-50 ${className}`}>
      <CardContent className="p-4 space-y-3">
        {/* Header with status */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {isRecording ? (
              <>
                <Mic className="h-4 w-4 text-red-500 animate-pulse" />
                <span className="text-sm font-medium">Recording...</span>
              </>
            ) : (
              <>
                <CheckCircle2 className="h-4 w-4 text-green-500" />
                <span className="text-sm font-medium">Complete</span>
              </>
            )}
          </div>
          {totalChunks && (
            <Badge variant="outline" className="text-xs">
              {processedChunks} / {totalChunks} chunks
            </Badge>
          )}
        </div>

        {/* Progress indicator */}
        {totalChunks && (
          <div className="w-full bg-muted rounded-full h-2">
            <div
              className="bg-primary h-2 rounded-full transition-all duration-300"
              style={{
                width: `${(processedChunks / totalChunks) * 100}%`,
              }}
            />
          </div>
        )}

        {/* Transcription results */}
        <div className="space-y-2 max-h-64 overflow-y-auto">
          {partialResults.length === 0 ? (
            <div className="flex items-center justify-center py-4 text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
              <span className="text-sm">Waiting for audio...</span>
            </div>
          ) : (
            partialResults.map((result) => (
              <div
                key={result.chunk_index}
                className={`p-2 rounded-md transition-all ${
                  result.is_final
                    ? "bg-green-50 dark:bg-green-950/20"
                    : "bg-muted"
                }`}
              >
                <div className="flex items-start justify-between gap-2 mb-1">
                  <span className="text-xs text-muted-foreground">
                    Chunk {result.chunk_index + 1}
                  </span>
                  <div className="flex items-center gap-2">
                    {/* Confidence indicator */}
                    <div className="flex items-center gap-1">
                      <div className="w-12 bg-muted rounded-full h-1.5 overflow-hidden">
                        <div
                          className={`h-full transition-all ${
                            result.confidence >= 0.8
                              ? "bg-green-500"
                              : result.confidence >= 0.6
                              ? "bg-yellow-500"
                              : "bg-red-500"
                          }`}
                          style={{ width: `${result.confidence * 100}%` }}
                        />
                      </div>
                      <span className="text-xs text-muted-foreground">
                        {(result.confidence * 100).toFixed(0)}%
                      </span>
                    </div>
                    {result.is_final && (
                      <Badge variant="outline" className="text-xs">
                        Final
                      </Badge>
                    )}
                  </div>
                </div>
                <p className="text-sm">
                  {result.text || (
                    <span className="text-muted-foreground italic">
                      Processing...
                    </span>
                  )}
                </p>
              </div>
            ))
          )}
        </div>

        {/* Full text preview */}
        {partialResults.length > 0 && (
          <div className="pt-3 border-t">
            <div className="text-xs text-muted-foreground mb-1">Full Text:</div>
            <p className="text-sm bg-muted p-2 rounded-md max-h-24 overflow-y-auto">
              {partialResults.map((r) => r.text).join(" ") || (
                <span className="text-muted-foreground italic">
                  No text yet...
                </span>
              )}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
