import { droid, modUrls, workerStrings } from 'https://esm.sh/@kbve/droid@0.0.3/es2022/droid.mjs';
import * as comlink from 'https://esm.sh/comlink';

(async () => {
  console.log('[DROID] init Library');

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
