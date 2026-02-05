import { useClientStore } from "@/stores/client";
import { useQuery } from "@pinia/colada";

type Sensors = {
  id: number;
  name: string;
  location: string;
  unit: string;
  latest_measurement: {
    temperature: number;
    cpu?: number;
    time: string;
  };
};

export function useSensors() {
  const clientStore = useClientStore();
  const query = useQuery({
    key: () => ["sensors"],
    query: () => clientStore.client("/api/sensors"),
  });

  return query;
}
