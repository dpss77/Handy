# Phase 2 UI Implementation

This document details the user interface components implemented for Phase 2 features: streaming transcription display and hot-plug device notifications.

## Overview

Phase 2 UI adds real-time visual feedback for:
- **Streaming Transcription**: Live display of partial transcription results during recording
- **Hot-Plug Notifications**: Toast notifications when audio devices connect/disconnect

## Components

### 1. Streaming Transcription UI

**Location**: `src/components/transcription/StreamingTranscription.tsx`

**Description**: A floating overlay that appears during recording to show real-time transcription progress.

**Features**:
- **Live Progress Tracking**: Shows chunk-by-chunk processing progress
- **Confidence Indicators**: Color-coded confidence meters for each chunk (green ≥80%, yellow ≥60%, red <60%)
- **Partial Results Display**: Shows text as it's transcribed, not just final result
- **Status Indicators**: Visual feedback for recording vs. complete states
- **Full Text Preview**: Concatenated view of all partial results
- **Auto-dismiss**: Hides 2 seconds after recording stops

**Visual Elements**:
```
┌─────────────────────────────────────┐
│ 🎤 Recording...      [3/5 chunks]   │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│                                     │
│ Chunk 1                      [███]  │
│ "Hello this is a test"       85%   │
│                                     │
│ Chunk 2                      [██─]  │
│ "of the streaming"           67%   │
│                                     │
│ Full Text:                          │
│ "Hello this is a test of the        │
│  streaming transcription..."        │
└─────────────────────────────────────┘
```

**Props**:
- `isActive` (boolean): Controls visibility of the overlay
- `className` (string, optional): Additional CSS classes

**Events Listened To**:
- `partial-transcription`: Receives partial transcription chunks from backend

**Backend Integration**:
The backend emits events with this structure:
```typescript
interface PartialTranscription {
  chunk_index: number;        // Zero-based chunk index
  total_chunks?: number;      // Total expected chunks (if known)
  text: string;               // Transcribed text for this chunk
  confidence: number;         // 0.0-1.0 confidence score
  is_final: boolean;          // True if this is the last chunk
}
```

### 2. Hot-Plug Notifications Hook

**Location**: `src/hooks/useHotplugNotifications.ts`

**Description**: React hook that listens for audio device connection changes and displays toast notifications.

**Features**:
- **Connect Notifications**: Green success toast when devices connect
- **Disconnect Notifications**: Red error toast when devices disconnect
- **Device Type Labels**: Shows "Microphone" or "Speaker" based on device type
- **Default Device Indication**: Appends "(Default)" for system default devices
- **Auto-dismiss**: Toasts disappear after 3 seconds

**Visual Examples**:

Connect notification:
```
✓ Microphone Connected
  Blue Yeti USB Microphone (Default)
```

Disconnect notification:
```
✗ Speaker Disconnected
  AirPods Pro
```

**Events Listened To**:
- `hotplug-event`: Receives device connection/disconnection events

**Backend Integration**:
The backend emits events with this structure:
```typescript
interface HotplugEvent {
  type: 'connected' | 'disconnected';
  device: {
    name: string;         // Device name
    is_input: boolean;    // true for microphones, false for speakers
    is_default: boolean;  // true if system default device
  };
}
```

### 3. Shadcn UI Components

To support the Ollama settings and streaming UI, we created lightweight implementations of shadcn components:

**Created Components**:
- `src/components/ui/card.tsx`: Card container components (Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter)
- `src/components/ui/badge.tsx`: Badge component with variants (default, secondary, destructive, outline)
- `src/components/ui/button.tsx`: Button component with variants and sizes
- `src/components/ui/input.tsx`: Input field component
- `src/components/ui/label.tsx`: Label component for form fields
- `src/components/ui/switch.tsx`: Toggle switch component
- `src/components/ui/select.tsx`: Select dropdown component (Select, SelectTrigger, SelectValue, SelectContent, SelectItem)
- `src/components/ui/textarea.tsx`: Multi-line text input component

**Design System**:
- Components use Tailwind CSS classes for styling
- Follow accessibility best practices (ARIA attributes, keyboard navigation)
- Support dark mode through CSS variables
- Consistent with existing Handy UI patterns

## Integration

### App.tsx Changes

The main application file was updated to integrate the new features:

```typescript
import { StreamingTranscription } from "./components/transcription/StreamingTranscription";
import { useHotplugNotifications } from "./hooks/useHotplugNotifications";

function App() {
  const [isStreamingActive, setIsStreamingActive] = useState(false);

  // Enable hot-plug notifications globally
  useHotplugNotifications();

  // Listen for recording events
  useEffect(() => {
    const unlistenStart = listen("recording-started", () => {
      setIsStreamingActive(true);
    });

    const unlistenEnd = listen("recording-stopped", () => {
      setTimeout(() => setIsStreamingActive(false), 2000);
    });

    return () => {
      unlistenStart.then(fn => fn());
      unlistenEnd.then(fn => fn());
    };
  }, []);

  return (
    <div className="h-screen flex flex-col">
      <Toaster />
      <StreamingTranscription isActive={isStreamingActive} />
      {/* ... rest of app ... */}
    </div>
  );
}
```

## Backend Event System

### Event Emissions

**Audio Manager** (`src-tauri/src/managers/audio.rs`):
- Emits `recording-started` when recording begins
- Emits `recording-stopped` when recording ends

**Streaming Processor** (`src-tauri/src/streaming/mod.rs`):
- Emits `partial-transcription` for each processed chunk
- Already implemented in Phase 2 backend work

**Hot-Plug Detector** (`src-tauri/src/lib.rs`):
- Initializes hot-plug detector on app startup
- Emits `hotplug-event` for device connect/disconnect
- Polling interval: 2 seconds (configurable)

### Event Flow

```
User presses shortcut
        ↓
AudioManager.try_start_recording()
        ↓
Emit: recording-started
        ↓
Frontend shows StreamingTranscription overlay
        ↓
Audio chunks processed
        ↓
Emit: partial-transcription (multiple times)
        ↓
Frontend updates StreamingTranscription display
        ↓
Recording stops
        ↓
Emit: recording-stopped
        ↓
Frontend hides StreamingTranscription after 2s
```

## User Experience

### Before Phase 2 UI:
- No visual feedback during recording except overlay spinner
- No indication when audio devices connect/disconnect
- User must wait until completion to see results

### After Phase 2 UI:
- **Immediate Feedback**: See transcription as you speak
- **Progress Tracking**: Know how much is processed vs. remaining
- **Confidence Awareness**: See which parts need review (low confidence)
- **Device Awareness**: Get notified about audio device changes
- **Better Workflow**: Can verify transcription quality in real-time

## Settings Integration

### Future Enhancement: Enable/Disable Controls

While not implemented in this phase, these settings would be natural additions:

```typescript
// In SettingsSchema (src/lib/types.ts)
export const SettingsSchema = z.object({
  // ... existing settings ...

  // Phase 2 UI settings (future)
  enable_streaming_display: z.boolean().optional().default(true),
  enable_hotplug_notifications: z.boolean().optional().default(true),
});
```

Users could toggle these features in Advanced Settings:
- "Show Streaming Transcription": Enable/disable the live transcription overlay
- "Device Connection Notifications": Enable/disable hot-plug toasts

## Performance Considerations

### Streaming Transcription:
- **Minimal Overhead**: Uses React state updates only when new data arrives
- **No Polling**: Event-driven architecture prevents unnecessary renders
- **Auto-cleanup**: Removes event listeners when component unmounts
- **Efficient Updates**: Only re-renders changed chunks, not entire display

### Hot-Plug Notifications:
- **Background Polling**: Runs in separate thread (Rust side)
- **No UI Blocking**: All device scanning happens off main thread
- **Configurable Interval**: Default 2s can be adjusted if needed
- **Cleanup**: Properly removes event listeners on unmount

## Testing

### Manual Testing Checklist:

**Streaming Transcription**:
- [ ] Overlay appears when recording starts
- [ ] Partial results display as they arrive
- [ ] Confidence bars show correct colors
- [ ] Progress bar advances correctly
- [ ] Full text preview concatenates all chunks
- [ ] Overlay disappears 2s after recording stops
- [ ] Multiple recordings in sequence work correctly

**Hot-Plug Notifications**:
- [ ] Toast appears when USB microphone connected
- [ ] Toast appears when Bluetooth headphones connected
- [ ] Toast appears when device disconnected
- [ ] Default device shows "(Default)" label
- [ ] Notifications don't interfere with recording
- [ ] Multiple device changes handled correctly

**Integration**:
- [ ] Both features work simultaneously
- [ ] No console errors during normal operation
- [ ] Event listeners clean up properly (no memory leaks)
- [ ] Settings window remains responsive
- [ ] Works across all supported platforms (macOS, Windows, Linux)

## Known Limitations

1. **Streaming Chunk Size**: Configured on backend (default 3s chunks). UI adapts but cannot control chunk size.

2. **Hot-Plug Polling**: Uses 2s polling interval. True hot-plug events would require platform-specific APIs (future enhancement).

3. **Position**: Streaming overlay is fixed to top-right. User customization not implemented.

4. **Streaming Accuracy**: Partial results may differ from final result due to context window. This is expected behavior.

## Files Modified/Created

### Frontend Files Created:
- `src/components/transcription/StreamingTranscription.tsx` (150 lines)
- `src/hooks/useHotplugNotifications.ts` (30 lines)
- `src/components/ui/card.tsx` (80 lines)
- `src/components/ui/badge.tsx` (25 lines)
- `src/components/ui/button.tsx` (50 lines)
- `src/components/ui/input.tsx` (25 lines)
- `src/components/ui/label.tsx` (20 lines)
- `src/components/ui/switch.tsx` (50 lines)
- `src/components/ui/select.tsx` (120 lines)
- `src/components/ui/textarea.tsx` (25 lines)

### Frontend Files Modified:
- `src/App.tsx` (added streaming state and hot-plug hook)

### Backend Files Modified:
- `src-tauri/src/lib.rs` (added hot-plug detector initialization)
- `src-tauri/src/managers/audio.rs` (added recording event emissions)

## Next Steps

Potential enhancements for Phase 3:
1. **Customizable Overlay Position**: Let users choose top-left, top-right, bottom-left, bottom-right
2. **Overlay Themes**: Match overlay styling to system theme
3. **Confidence Threshold Warning**: Alert if too many low-confidence chunks
4. **Real Hot-Plug**: Platform-specific implementations for true event-driven detection
5. **Settings Controls**: Add enable/disable toggles in Advanced Settings
6. **Notification Preferences**: Customize which device types trigger notifications

## Summary

Phase 2 UI successfully brings real-time feedback to Handy users:
- **400+ lines** of new frontend code
- **Zero breaking changes** to existing functionality
- **Event-driven architecture** ensures minimal performance impact
- **Comprehensive shadcn components** for future UI development

These features transform Handy from a "record and wait" tool to a "record and see" experience, significantly improving user confidence and workflow efficiency.
