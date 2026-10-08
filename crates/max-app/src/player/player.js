// Runs in the embedded player. The web player's own back button, and whatever it does when a
// video ends, lead to pages of the web app; this hands control back to the native window instead.
(() => {
  if (window.top !== window) return;
  // Dark from the first paint, and no page scrollbar beside the video.
  const style = document.createElement('style');
  style.textContent = 'html { background: #000 !important; } ::-webkit-scrollbar { display: none; }';
  const addStyle = () => (document.head || document.documentElement).appendChild(style);
  if (document.documentElement) addStyle();
  else document.addEventListener('readystatechange', addStyle, { once: true });

  let closed = false;
  const close = () => {
    if (closed) return;
    closed = true;
    // Hide the page at once so the web app's next screen never shows.
    document.documentElement.style.visibility = 'hidden';
    document.querySelectorAll('video, audio').forEach((media) => media.pause());
    window.ipc.postMessage('close');
  };
  // Leaving a playback route by any means closes the player.
  let playing = false;
  const check = () => {
    if (location.pathname.includes('/watch')) playing = true;
    else if (playing) close();
  };
  setInterval(check, 150);
  addEventListener('popstate', check);
  // Only videos belong here. If the site sends this view anywhere else instead (a sign-in or
  // error page, say), give up and return rather than show a page of the web app.
  setTimeout(() => { if (!playing) close(); }, 6000);
  // Escape leaves the player, unless it is being used to leave full screen.
  addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !document.fullscreenElement && !document.webkitFullscreenElement) close();
  }, true);
  if (window.__maxrsMuted) {
    const play = HTMLMediaElement.prototype.play;
    HTMLMediaElement.prototype.play = function () {
      this.muted = true;
      return play.apply(this, arguments);
    };
  }
})();
