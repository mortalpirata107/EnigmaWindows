import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { NetworkDetails } from '../types';

const defaultNetwork: NetworkDetails = {
  conn_type: 'ethernet',
  state: 'connected',
  device: 'enp4s0',
  name: 'Conexión cableada 1',
  speed_label: '1.0 Gbps',
  ip_address: '',
  wifi_networks: [],
};

let cachedNetwork: NetworkDetails = defaultNetwork;
let lastFetchTime = 0;
let inflightPromise: Promise<NetworkDetails> | null = null;

export function useDetailedNetwork() {
  const [network, setNetwork] = useState<NetworkDetails>(cachedNetwork);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const fetchNetwork = useCallback(async () => {
    const now = Date.now();
    if (now - lastFetchTime < 4000) {
      setNetwork(cachedNetwork);
      return;
    }
    if (inflightPromise) {
      const res = await inflightPromise;
      setNetwork(res);
      return;
    }
    try {
      inflightPromise = invoke<NetworkDetails>('get_detailed_network_status');
      const res = await inflightPromise;
      cachedNetwork = res;
      lastFetchTime = Date.now();
      setNetwork(res);
    } catch (err) {
      console.warn('[useDetailedNetwork] Error fetching network status:', err);
    } finally {
      inflightPromise = null;
    }
  }, []);

  const connectWifi = useCallback(async (ssid: string, password?: string) => {
    setIsLoading(true);
    try {
      await invoke('connect_wifi_network', { ssid, password: password || null });
      await fetchNetwork();
    } catch (err) {
      console.warn('[useDetailedNetwork] Error connecting wifi:', err);
    } finally {
      setIsLoading(false);
    }
  }, [fetchNetwork]);

  const disconnectCurrent = useCallback(async () => {
    if (!network.device) return;
    setIsLoading(true);
    try {
      await invoke('disconnect_network', { device: network.device });
      await fetchNetwork();
    } catch (err) {
      console.warn('[useDetailedNetwork] Error disconnecting network:', err);
    } finally {
      setIsLoading(false);
    }
  }, [network.device, fetchNetwork]);

  useEffect(() => {
    fetchNetwork();
    const interval = setInterval(fetchNetwork, 18000);
    return () => clearInterval(interval);
  }, [fetchNetwork]);

  return {
    network,
    isLoading,
    fetchNetwork,
    connectWifi,
    disconnectCurrent,
  };
}
