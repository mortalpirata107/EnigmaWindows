import { invoke } from '@tauri-apps/api/core';

let lastTopBarRegionsKey = '';

function q(val: number): number {
  return Math.round(val / 4) * 4;
}

export function getTopBarRegions(): [number, number, number, number][] {
  const dpr = window.devicePixelRatio || 1;
  const regions: [number, number, number, number][] = [];
  const screenW = window.innerWidth;
  const centerX = screenW / 2;

  if (document.body.dataset.modalOpen === 'true') {
    const screenH = window.innerHeight;
    return [[0, 0, Math.round(screenW * dpr), Math.round(screenH * dpr)]];
  }

  // 1. Left Capsule (floating island)
  const leftEl = document.getElementById('left-capsule');
  if (leftEl) {
    const r = leftEl.getBoundingClientRect();
    const x = Math.max(0, r.left);
    const y = Math.max(0, r.top);
    const w = r.width > 50 ? r.width : 280;
    const h = r.height > 20 ? r.height : 36;
    regions.push([
      q(x * dpr),
      q(y * dpr),
      q(w * dpr),
      q(h * dpr),
    ]);
  } else {
    regions.push([q(12 * dpr), q(14 * dpr), q(280 * dpr), q(36 * dpr)]);
  }

  // 2. Center Dynamic Notch and Enigma Forcefield Container (floating dynamic island)
  const notchEl = document.getElementById('dynamic-notch');
  const enigmaInput = document.getElementById('enigma-input-capsule');
  const enigmaOutput = document.getElementById('enigma-output-panel');
  const forcefieldEl = document.getElementById('enigma-forcefield-container');

  if (enigmaInput || enigmaOutput) {
    // Enigma AI está activo: otorgar hitbox al contenedor padre completo y todos sus hijos activos
    const targetEl = forcefieldEl || notchEl;
    if (targetEl) {
      const r = targetEl.getBoundingClientRect();
      let minLeft = r.width > 50 ? r.left : (centerX - 320);
      let maxRight = r.width > 50 ? r.right : (centerX + 320);
      let maxBottom = r.bottom;

      if (enigmaOutput) {
        const outR = enigmaOutput.getBoundingClientRect();
        if (outR.bottom > maxBottom) maxBottom = outR.bottom;
        if (outR.left < minLeft && outR.left > 0) minLeft = outR.left;
        if (outR.right > maxRight) maxRight = outR.right;
      }
      if (enigmaInput) {
        const inR = enigmaInput.getBoundingClientRect();
        if (inR.bottom > maxBottom) maxBottom = inR.bottom;
        if (inR.left < minLeft && inR.left > 0) minLeft = inR.left;
        if (inR.right > maxRight) maxRight = inR.right;
      }

      const top = Math.max(0, r.top);
      const w = Math.max(360, maxRight - minLeft + 16);
      const h = Math.max(300, maxBottom - top + 16);
      const left = Math.max(0, minLeft - 8);

      regions.push([
        Math.max(0, q(left * dpr)),
        q(top * dpr),
        q(w * dpr),
        q(h * dpr),
      ]);
    }
  } else if (notchEl) {
    const isExpanded = notchEl.dataset.expanded === 'true';
    const isSearch = notchEl.dataset.search === 'true';
    const section = notchEl.dataset.section;
    const r = notchEl.getBoundingClientRect();
    const top = Math.max(0, r.top);

    const isClipboard = notchEl.dataset.clipboard === 'true';

    if (isClipboard) {
      const targetW = 560;
      const targetH = 490;
      const left = centerX - targetW / 2;
      regions.push([
        Math.max(0, q(left * dpr)),
        q(top * dpr),
        q(targetW * dpr),
        q(targetH * dpr),
      ]);
    } else if (isExpanded) {
      // Cobertura completa inmediata del notch expandido a t=0 para evitar que el ratón salga durante la animación
      const targetW = isSearch ? 560 : Math.max(520, r.width > 100 ? r.width : 520);
      const targetH = isSearch ? 430 : (section === 'settings' ? 360 : (section === 'telemetry' ? 220 : Math.max(300, r.height > 100 ? r.height : 300)));
      const left = centerX - targetW / 2;
      regions.push([
        Math.max(0, q(left * dpr)),
        q(top * dpr),
        q(targetW * dpr),
        q(targetH * dpr),
      ]);
    } else {
      // En reposo: Notch dinámico flotante
      const defaultW = Math.max(200, r.width > 50 ? r.width : 240);
      const defaultH = r.height > 20 ? r.height : 36;
      const left = r.width > 50 ? r.left : (centerX - defaultW / 2);
      regions.push([
        Math.max(0, q(left * dpr)),
        q(top * dpr),
        q(defaultW * dpr),
        q(defaultH * dpr),
      ]);
    }
  } else {
    regions.push([
      q((centerX - 120) * dpr),
      q(14 * dpr),
      q(240 * dpr),
      q(36 * dpr),
    ]);
  }

  // 3. Right Status Capsule (floating island)
  const rightEl = document.getElementById('right-status-capsule');
  if (rightEl) {
    const r = rightEl.getBoundingClientRect();
    const expandedChild = rightEl.querySelector('[data-menu-expanded="true"]');
    const top = Math.max(0, r.top);
    
    if (expandedChild) {
      const cRect = expandedChild.getBoundingClientRect();
      const minX = Math.min(r.left > 50 ? r.left : (screenW - 380), cRect.left, r.right - 380);
      const maxX = Math.max(r.right > 50 ? r.right : screenW, cRect.right);
      const maxY = Math.max(cRect.bottom + 8, 40);
      regions.push([
        q(minX * dpr),
        q(top * dpr),
        q((maxX - minX) * dpr),
        q(maxY * dpr),
      ]);
    } else {
      const minX = r.width > 50 ? r.left : (screenW - 280);
      const w = r.width > 50 ? r.width : 280;
      const h = r.height > 20 ? r.height : 36;
      regions.push([
        q(minX * dpr),
        q(top * dpr),
        q(w * dpr),
        q(h * dpr),
      ]);
    }
  } else {
    regions.push([q((screenW - 290) * dpr), q(14 * dpr), q(280 * dpr), q(36 * dpr)]);
  }

  return regions;
}

export function forceSyncTopBarRegions() {
  lastTopBarRegionsKey = '';
  syncAllTopBarRegions();
}

export function syncAllTopBarRegions() {
  try {
    const isModalActive = document.body.dataset.modalOpen === 'true';

    const regions = getTopBarRegions();
    const key = JSON.stringify(regions);
    if (lastTopBarRegionsKey === key) return;
    lastTopBarRegionsKey = key;

    if (isModalActive) {
      invoke('log_frontend', {
        msg: `[syncAllTopBarRegions] Modal active - applied full input pass: ${key}`,
      }).catch(() => {});
    }

    invoke('update_custom_input_regions', { regions }).catch(() => {});
  } catch (err) {
    console.warn('[syncAllTopBarRegions] Error:', err);
  }
}
