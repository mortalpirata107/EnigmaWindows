import React from 'react';
import { 
  Bluetooth, 
  Power, 
  CheckCircle2, 
  Link2, 
  Trash2, 
  PlusCircle, 
  Radio
} from 'lucide-react';
import { useDetailedBluetooth } from '../hooks/useDetailedBluetooth';

export const BluetoothContent: React.FC = () => {
  const { 
    state, 
    isLoading, 
    disconnectDevice, 
    unpairDevice, 
    pairDevice, 
    togglePower 
  } = useDetailedBluetooth();

  return (
    <div 
      className="w-full h-full flex flex-col p-3.5 text-white select-none overflow-hidden"
      onClick={(e) => e.stopPropagation()}
    >
      {/* 1. Header con Interruptor de Encendido/Apagado */}
      <div className="flex items-center justify-between pb-2.5 mb-2.5 border-b border-[#EF4444]/20 shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-[#08080C] border border-[#EF4444]/50 flex items-center justify-center text-[#EF4444] shadow-[0_0_8px_rgba(239,68,68,0.3)]">
            <Bluetooth className="w-4 h-4 text-[#EF4444]" />
          </div>
          <div>
            <span className="text-xs font-bold font-mono tracking-wider text-white uppercase block">
              Bluetooth
            </span>
            <span className="text-[10px] text-white/50 block font-mono">
              {state.powered ? 'Activado y visible' : 'Apagado'}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => togglePower(!state.powered)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-mono font-bold transition-all cursor-pointer ${
              state.powered
                ? 'bg-[#EF4444] border border-[#EF4444] text-white shadow-[0_0_10px_rgba(239,68,68,0.5)]'
                : 'bg-[#08080C] border border-white/20 text-white/60 hover:text-white'
            }`}
          >
            <Power className={`w-3 h-3 ${state.powered ? 'text-white' : 'text-white/40'}`} />
            <span>{state.powered ? 'ON' : 'OFF'}</span>
          </button>
        </div>
      </div>

      {/* Contenido Scrollable */}
      <div className="flex-1 overflow-y-auto space-y-3 pr-1 custom-scrollbar min-h-0">
        {!state.powered ? (
          <div className="text-center py-8">
            <Bluetooth className="w-10 h-10 text-[#EF4444]/40 mx-auto mb-2" />
            <p className="text-xs text-white/60 font-mono">El Bluetooth está desactivado</p>
            <p className="text-[11px] text-white/40 mt-1">Actívalo para conectar auriculares y periféricos</p>
          </div>
        ) : (
          <>
            {/* SECCIÓN 1: DISPOSITIVOS CONECTADOS */}
            <div>
              <div className="flex items-center justify-between mb-1.5 px-1">
                <span className="text-[11px] font-mono font-bold text-[#EF4444] uppercase tracking-wider flex items-center gap-1.5">
                  <CheckCircle2 className="w-3 h-3 text-[#EF4444]" />
                  Conectados ({state.connected_devices.length})
                </span>
              </div>
              <div className="space-y-1.5">
                {state.connected_devices.length === 0 ? (
                  <div className="px-3 py-2 rounded-xl bg-[#08080C] border border-white/10 text-[11px] text-white/40 font-mono">
                    Ningún dispositivo conectado actualmente
                  </div>
                ) : (
                  state.connected_devices.map((dev) => (
                    <div
                      key={dev.mac}
                      className="flex items-center justify-between px-3 py-2 rounded-xl bg-[#08080C] border border-[#EF4444]/40 shadow-[0_0_8px_rgba(239,68,68,0.15)]"
                    >
                      <div className="min-w-0 pr-2">
                        <span className="text-xs font-semibold text-white block truncate">
                          {dev.name}
                        </span>
                        <span className="text-[10px] font-mono text-white/40 block truncate">
                          {dev.mac}
                        </span>
                      </div>
                      <button
                        type="button"
                        onClick={() => disconnectDevice(dev.mac)}
                        disabled={isLoading}
                        className="px-2.5 py-1 rounded-lg bg-[#08080C] border border-[#EF4444]/40 text-[10px] font-mono font-bold text-[#EF4444] hover:bg-[#EF4444]/20 hover:text-white transition-all cursor-pointer shrink-0"
                      >
                        Desconectar
                      </button>
                    </div>
                  ))
                )}
              </div>
            </div>

            {/* SECCIÓN 2: DISPOSITIVOS EMPAREJADOS (NO CONECTADOS) */}
            <div>
              <div className="flex items-center justify-between mb-1.5 px-1">
                <span className="text-[11px] font-mono font-bold text-white/70 uppercase tracking-wider flex items-center gap-1.5">
                  <Link2 className="w-3 h-3 text-[#EF4444]" />
                  Emparejados ({state.paired_devices.length})
                </span>
              </div>
              <div className="space-y-1.5">
                {state.paired_devices.length === 0 ? (
                  <div className="px-3 py-2 rounded-xl bg-[#08080C] border border-white/10 text-[11px] text-white/40 font-mono">
                    No hay dispositivos guardados
                  </div>
                ) : (
                  state.paired_devices.map((dev) => (
                    <div
                      key={dev.mac}
                      className="flex items-center justify-between px-3 py-2 rounded-xl bg-[#08080C] border border-white/10 hover:border-[#EF4444]/30 transition-colors"
                    >
                      <div className="min-w-0 pr-2">
                        <span className="text-xs font-medium text-white block truncate">
                          {dev.name}
                        </span>
                        <span className="text-[10px] font-mono text-white/40 block truncate">
                          {dev.mac}
                        </span>
                      </div>
                      <div className="flex items-center gap-1.5 shrink-0">
                        <button
                          type="button"
                          onClick={() => unpairDevice(dev.mac)}
                          disabled={isLoading}
                          title="Desemparejar dispositivo"
                          className="p-1 rounded-lg bg-[#08080C] border border-white/10 text-white/50 hover:text-[#EF4444] hover:border-[#EF4444]/40 transition-all cursor-pointer"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                        <button
                          type="button"
                          onClick={() => pairDevice(dev.mac)}
                          disabled={isLoading}
                          className="px-2.5 py-1 rounded-lg bg-[#08080C] border border-[#EF4444]/50 text-[10px] font-mono font-bold text-white hover:bg-[#EF4444]/30 transition-all cursor-pointer"
                        >
                          Conectar
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>

            {/* SECCIÓN 3: DISPOSITIVOS DISPONIBLES */}
            <div>
              <div className="flex items-center justify-between mb-1.5 px-1">
                <span className="text-[11px] font-mono font-bold text-white/70 uppercase tracking-wider flex items-center gap-1.5">
                  <Radio className="w-3 h-3 text-[#EF4444]" />
                  Disponibles ({state.available_devices.length})
                </span>
              </div>
              <div className="space-y-1.5">
                {state.available_devices.length === 0 ? (
                  <div className="px-3 py-2 rounded-xl bg-[#08080C] border border-white/10 text-[11px] text-white/40 font-mono">
                    Buscando dispositivos cercanos...
                  </div>
                ) : (
                  state.available_devices.map((dev) => (
                    <div
                      key={dev.mac}
                      className="flex items-center justify-between px-3 py-2 rounded-xl bg-[#08080C] border border-white/10 hover:border-[#EF4444]/30 transition-colors"
                    >
                      <div className="min-w-0 pr-2">
                        <span className="text-xs font-medium text-white block truncate">
                          {dev.name || 'Dispositivo Desconocido'}
                        </span>
                        <span className="text-[10px] font-mono text-white/40 block truncate">
                          {dev.mac}
                        </span>
                      </div>
                      <button
                        type="button"
                        onClick={() => pairDevice(dev.mac)}
                        disabled={isLoading}
                        className="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-[#08080C] border border-[#EF4444]/40 text-[10px] font-mono font-bold text-[#EF4444] hover:bg-[#EF4444]/20 hover:text-white transition-all cursor-pointer shrink-0"
                      >
                        <PlusCircle className="w-3 h-3 text-[#EF4444]" />
                        <span>Emparejar</span>
                      </button>
                    </div>
                  ))
                )}
              </div>
            </div>
          </>
        )}
      </div>

      {/* Margen inferior mínimo 5px */}
      <div className="h-1 shrink-0" />
    </div>
  );
};
