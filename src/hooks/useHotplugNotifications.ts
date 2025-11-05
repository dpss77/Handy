import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { HotplugEvent } from "@/lib/types";

/**
 * Hook that listens for audio device hotplug events and displays toast notifications
 * when devices are connected or disconnected.
 */
export function useHotplugNotifications() {
  useEffect(() => {
    // Listen for hotplug events from the backend
    const unlistenHotplug = listen<HotplugEvent>("hotplug-event", (event) => {
      const hotplugEvent = event.payload;
      const deviceType = hotplugEvent.device.is_input ? "Microphone" : "Speaker";
      const isDefault = hotplugEvent.device.is_default ? " (Default)" : "";

      if (hotplugEvent.type === "connected") {
        toast.success(`${deviceType} Connected`, {
          description: `${hotplugEvent.device.name}${isDefault}`,
          duration: 3000,
        });
      } else if (hotplugEvent.type === "disconnected") {
        toast.error(`${deviceType} Disconnected`, {
          description: `${hotplugEvent.device.name}${isDefault}`,
          duration: 3000,
        });
      }
    });

    return () => {
      unlistenHotplug.then((fn) => fn());
    };
  }, []);
}
