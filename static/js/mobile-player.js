/* Cataclyse 11L.1 — Mode cinéma iPhone */
(() => {
    'use strict';

    if (!/iPhone|iPod/i.test(navigator.userAgent)) return;

    const modal = document.getElementById('playerModal');
    if (!modal) return;

    let wasOpen = false;
    let portraitRequested = false;

    const portraitViewport = () =>
        window.matchMedia(
            '(orientation: portrait) and (max-width: 640px)'
        ).matches;

    function isScanOpen() {
        const scan = document.getElementById(
            'netflixScanViewport'
        );

        return !!scan &&
            getComputedStyle(scan).display !== 'none';
    }

    function synchronizeCinema() {
        const opened = modal.classList.contains('open');

        if (opened && !wasOpen) {
            portraitRequested = false;
        }

        if (!opened) {
            portraitRequested = false;
        }

        wasOpen = opened;

        const rotate = opened &&
            portraitViewport() &&
            !portraitRequested &&
            !isScanOpen();

        modal.classList.toggle(
            'cat-cinema-rotated',
            rotate
        );
    }

    const observer = new MutationObserver(
        synchronizeCinema
    );

    observer.observe(modal, {
        attributes: true,
        attributeFilter: ['class']
    });

    // Le bouton plein écran permet de changer
    // entre le mode cinéma et le mode portrait.
    document.addEventListener('click', event => {
        if (!(event.target instanceof Element)) return;

        const button = event.target.closest(
            '#netflixFullscreenBtn'
        );

        if (
            !button ||
            !modal.classList.contains('open')
        ) return;

        if (!portraitViewport()) {
            // En paysage réel, conserver le comportement
            // plein écran normal du navigateur.
            return;
        }

        event.preventDefault();
        event.stopImmediatePropagation();

        portraitRequested = !portraitRequested;

        synchronizeCinema();

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    }, true);

    window.addEventListener(
        'resize',
        synchronizeCinema,
        { passive: true }
    );

    window.addEventListener(
        'orientationchange',
        synchronizeCinema,
        { passive: true }
    );

    synchronizeCinema();
})();


/* Cataclyse 11L.2 : réglage Remplir / Ajuster */
(() => {
    'use strict';

    if (!/iPhone|iPod/i.test(navigator.userAgent)) return;

    const modal = document.getElementById('playerModal');
    const fullscreen = document.getElementById(
        'netflixFullscreenBtn'
    );

    if (!modal || !fullscreen ||
        document.getElementById('catCinemaFitBtn')) return;

    const button = document.createElement('button');
    button.id = 'catCinemaFitBtn';
    button.type = 'button';
    button.className = 'netflix-ctrl-btn';

    fullscreen.parentNode.insertBefore(button, fullscreen);

    let fill = true;

    function update() {
        modal.classList.toggle('cat-video-fill', fill);
        button.textContent = fill ? 'Ajuster' : 'Remplir';
        button.setAttribute('aria-pressed', String(fill));
        button.setAttribute(
            'aria-label',
            fill
                ? "Afficher l'image entière"
                : "Remplir l'écran avec la vidéo"
        );
        button.title = button.getAttribute('aria-label');
    }

    button.addEventListener('click', () => {
        fill = !fill;
        update();

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    });

    update();
})();


/* Cataclyse 11L.3 — Commandes tactiles et réveil */
(() => {
    'use strict';

    if (!/iPhone|iPod/i.test(navigator.userAgent)) return;

    const modal = document.getElementById('playerModal');
    const container = document.getElementById(
        'netflixPlayerContainer'
    );
    const shield = document.getElementById(
        'netflixClickShield'
    );

    if (!modal || !container || !shield) return;

    let suppressClick = false;
    let suppressTimer = null;

    const active = () =>
        modal.classList.contains('open');

    function wakeControls() {
        if (!active()) return;

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    }

    // Détecter le toucher avant que l'interface
    // ne perde son état « idle ».
    shield.addEventListener('pointerdown', event => {
        if (
            !active() ||
            event.pointerType !== 'touch' ||
            !container.classList.contains('idle')
        ) return;

        suppressClick = true;
        clearTimeout(suppressTimer);

        suppressTimer = setTimeout(() => {
            suppressClick = false;
        }, 1000);

        wakeControls();
    }, { capture: true, passive: true });

    // Le toucher qui réveille les contrôles
    // ne doit pas lancer play/pause.
    shield.addEventListener('click', event => {
        if (!suppressClick) return;

        suppressClick = false;
        clearTimeout(suppressTimer);

        event.preventDefault();
        event.stopImmediatePropagation();
    }, true);

    shield.addEventListener('pointercancel', () => {
        suppressClick = false;
        clearTimeout(suppressTimer);
    });

    // Un toucher sur les autres éléments du lecteur
    // réinitialise la période d'inactivité.
    container.addEventListener(
        'pointerdown',
        wakeControls,
        { passive: true }
    );

    window.addEventListener('pagehide', () => {
        suppressClick = false;
        clearTimeout(suppressTimer);
    });
})();


/* Cataclyse 11L.4 — Interactions tactiles iPhone */
(() => {
    'use strict';

    if (!/iPhone|iPod/i.test(navigator.userAgent)) return;

    const modal = document.getElementById('playerModal');
    const container = document.getElementById(
        'netflixPlayerContainer'
    );
    const shield = document.getElementById(
        'netflixClickShield'
    );
    const scrubber = document.getElementById(
        'netflixScrubberContainer'
    );

    if (!modal || !container || !shield || !scrubber) return;

    const playerOpen = () =>
        modal.classList.contains('open');

    // Premier toucher : afficher les commandes.
    // Bloquer uniquement le clic généré par ce toucher,
    // pour éviter une pause involontaire.
    let pendingClick = null;
    let pendingTimeout = null;

    function clearPendingClick() {
        if (pendingClick) {
            window.removeEventListener(
                'click', pendingClick, true
            );
            pendingClick = null;
        }
        clearTimeout(pendingTimeout);
        pendingTimeout = null;
    }

    window.addEventListener('pointerdown', event => {
        if (
            !playerOpen() ||
            event.pointerType !== 'touch' ||
            !container.classList.contains('idle') ||
            !shield.contains(event.target)
        ) return;

        clearPendingClick();

        pendingClick = click => {
            if (modal.contains(click.target)) {
                click.preventDefault();
                click.stopImmediatePropagation();
            }
            clearPendingClick();
        };

        window.addEventListener('click', pendingClick, true);

        pendingTimeout = setTimeout(
            clearPendingClick, 900
        );

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    }, true);

    // Le lecteur tourne de +90 degrés.
    // Dans cette orientation, l'axe horizontal du lecteur
    // correspond à l'axe vertical physique de l'iPhone.
    function progressFraction(event) {
        const rect = scrubber.getBoundingClientRect();
        const rotated = modal.classList.contains(
            'cat-cinema-rotated'
        );

        const position = rotated
            ? event.clientY - rect.top
            : event.clientX - rect.left;

        const length = rotated
            ? rect.height
            : rect.width;

        return Math.max(
            0,
            Math.min(1, position / Math.max(length, 1))
        );
    }

    // Gestion native des gestes tactiles sur la timeline.
    scrubber.style.touchAction = 'none';

    let activePointer = null;
    let legacyMouseBlockedUntil = 0;

    const tooltip = document.getElementById(
        'netflixScrubberTooltip'
    );
    const played = document.getElementById(
        'netflixProgressPlayed'
    );

    function preview(event) {
        const duration = Number(playerState.duration);

        if (!Number.isFinite(duration) || duration <= 0) {
            return;
        }

        const fraction = progressFraction(event);
        const seconds = fraction * duration;

        if (played) {
            played.style.width = `${fraction * 100}%`;
        }

        if (tooltip) {
            tooltip.style.left = `${fraction * 100}%`;
            tooltip.textContent = formatPlayerTime(seconds);
            tooltip.style.display = 'block';
        }

        renderTimeDisplay(seconds, duration);
    }

    scrubber.addEventListener('pointerdown', event => {
        if (
            !playerOpen() ||
            event.pointerType !== 'touch' ||
            activePointer !== null
        ) return;

        event.preventDefault();
        event.stopPropagation();

        activePointer = event.pointerId;
        legacyMouseBlockedUntil = Date.now() + 1200;
        playerState.isDraggingScrubber = true;

        try {
            scrubber.setPointerCapture(event.pointerId);
        } catch (_) {}

        preview(event);

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    }, true);

    scrubber.addEventListener('pointermove', event => {
        if (activePointer !== event.pointerId) return;

        event.preventDefault();
        preview(event);
    });

    function finishScrub(event, apply) {
        if (activePointer !== event.pointerId) return;

        const duration = Number(playerState.duration);
        const fraction = progressFraction(event);

        activePointer = null;
        playerState.isDraggingScrubber = false;

        if (tooltip) tooltip.style.display = 'none';

        try {
            if (scrubber.hasPointerCapture(event.pointerId)) {
                scrubber.releasePointerCapture(event.pointerId);
            }
        } catch (_) {}

        if (
            apply &&
            Number.isFinite(duration) &&
            duration > 0 &&
            typeof seekTo === 'function'
        ) {
            seekTo(fraction * duration);
        }

        if (typeof resetIdleTimer === 'function') {
            resetIdleTimer();
        }
    }

    scrubber.addEventListener('pointerup', event => {
        finishScrub(event, true);
    });

    scrubber.addEventListener('pointercancel', event => {
        finishScrub(event, false);
    });

    // Éviter que les événements souris synthétiques
    // déclenchent aussi l'ancien mécanisme de scrubbing.
    scrubber.addEventListener('mousedown', event => {
        if (Date.now() < legacyMouseBlockedUntil) {
            event.preventDefault();
            event.stopImmediatePropagation();
        }
    }, true);

    window.addEventListener('pagehide', () => {
        clearPendingClick();
        activePointer = null;
        playerState.isDraggingScrubber = false;
    });
})();
