(() => {
    const state = {
        mode: 0,
        on: false,
        fan: 0,
        swing: false,
        temp: 24
    };
    const TEMP_MIN = 16,
        TEMP_MAX = 30;

    const elPower   = document.getElementById('btn-power');
    const elTempVal = document.getElementById('temp-value');
    const elTempDown = document.getElementById('btn-temp-down');
    const elTempUp  = document.getElementById('btn-temp-up');
    const elSwing   = document.getElementById('btn-swing');
    const elStatus  = document.getElementById('status');
    const modeBtns  = document.querySelectorAll('#mode-btns .mode-btn');
    const fanBtns   = document.querySelectorAll('#fan-btns .mode-btn');

    function render() {
        elPower.classList.toggle('on', state.on);
        elTempVal.textContent = state.temp;
        elTempDown.disabled = state.temp <= TEMP_MIN;
        elTempUp.disabled   = state.temp >= TEMP_MAX;
        modeBtns.forEach(b => b.classList.toggle('active', +b.dataset.value === state.mode));
        fanBtns.forEach(b  => b.classList.toggle('active', +b.dataset.value === state.fan));
        elSwing.classList.toggle('active', state.swing);
        elSwing.textContent = state.swing ? 'On' : 'Off';
    }

    async function sendCommand() {
        setStatus('Sending…', '');
        try {
            const res = await fetch('/command', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(state),
            });
            if (res.ok) {
                setStatus('Sent ✓', 'ok');
            } else {
                setStatus(`Error ${res.status}`, 'err');
            }
        } catch {
            setStatus('Network error', 'err');
        }
    }

    function setStatus(msg, cls) {
        elStatus.textContent = msg;
        elStatus.className = 'status ' + cls;
    }

    function act(mutateFn) {
        mutateFn();
        render();
        sendCommand();
    }

    elPower.addEventListener('click', () => act(() => { state.on = !state.on; }));

    elTempDown.addEventListener('click', () => {
        if (state.temp > TEMP_MIN) act(() => { state.temp--; });
    });
    elTempUp.addEventListener('click', () => {
        if (state.temp < TEMP_MAX) act(() => { state.temp++; });
    });

    modeBtns.forEach(b => b.addEventListener('click', () =>
        act(() => { state.mode = +b.dataset.value; })
    ));
    fanBtns.forEach(b => b.addEventListener('click', () =>
        act(() => { state.fan = +b.dataset.value; })
    ));

    elSwing.addEventListener('click', () => act(() => { state.swing = !state.swing; }));

    function init() {
        fetch('/state')
            .then(r => r.ok ? r.json() : Promise.reject(r.status))
            .then(s => { Object.assign(state, s); render(); })
            .catch(() => render());
    }

    // START_DEV
    function init() { render(); }
    // END_DEV

    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register('/sw.js').catch(() => {});
    }

    init();
})();
