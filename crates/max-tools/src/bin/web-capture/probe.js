// Runs in every page: report which DRM key systems this web view offers.
(async () => {
  if (window.top !== window) return;
  const out = { href: location.origin + location.pathname, ua: navigator.userAgent };
  const cfg = [{
    initDataTypes: ['sinf', 'skd', 'cenc'],
    videoCapabilities: [{ contentType: 'video/mp4; codecs="avc1.42E01E"' }],
  }];
  for (const ks of ['com.apple.fps', 'com.apple.fps.1_0', 'com.widevine.alpha']) {
    try {
      await navigator.requestMediaKeySystemAccess(ks, cfg);
      out[ks] = true;
    } catch (e) {
      out[ks] = String(e);
    }
  }
  out.legacyWebKitKeys = typeof WebKitMediaKeys !== 'undefined'
    && WebKitMediaKeys.isTypeSupported('com.apple.fps.1_0', 'video/mp4');
  window.ipc.postMessage(JSON.stringify(out));
})();
