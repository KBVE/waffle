import { droid, modUrls } from 'https://esm.sh/@kbve/droid';
import * as comlink from 'https://esm.sh/comlink';

(async () => {
  console.log('[DROID] init Library');

    const workerStrings = {
      canvasWorker: 'https://esm.sh/@kbve/droid/lib/workers/canvas-worker.js',
      dbWorker: 'https://esm.sh/@kbve/droid/lib/workers/db-worker.js',
      wsWorker: 'https://esm.sh/@kbve/droid/lib/workers/ws-worker.js',
    };

  await droid({ workerURLs: workerStrings });

  const mod = window.kbve?.mod;
  const emitFromWorker = window.kbve?.uiux?.emitFromWorker;

  if (!mod) {
    console.error('[KBVE] Mod manager not available');
    return;
  }

  const bentoMod = await mod.load(modUrls.bento);

  if (bentoMod?.instance?.init && typeof emitFromWorker === 'function') {
    await bentoMod.instance.init(comlink.proxy({ emitFromWorker }));
  }

  if (bentoMod?.meta) {
    window.kbve?.events?.emit('droid-mod-ready', {
      meta: bentoMod.meta,
      timestamp: Date.now(),
    });
  }

  console.log('[KBVE] Bento mod loaded');
})();
