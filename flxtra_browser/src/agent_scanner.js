(function() {
    let items = [];
    let idCounter = 1;

    function isVisible(el) {
        if (!el) return false;
        const style = window.getComputedStyle(el);
        if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
        const rect = el.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0;
    }

    function getLabel(el) {
        return el.innerText || el.placeholder || el.getAttribute('aria-label') || el.name || el.id || '';
    }

    // Scan interactive elements
    const selectors = [
        'a[href]',
        'button',
        'input:not([type="hidden"])',
        'textarea',
        'select',
        '[role="button"]',
        '[onclick]'
    ];

    document.querySelectorAll(selectors.join(',')).forEach(el => {
        if (!isVisible(el)) return;
        
        // Assign a temporary visual ID for the user to see (debugging/visual grounding)
        // In a real agent, we might overlay this. For now, we just track it internally.
        
        let label = getLabel(el).trim().slice(0, 50); // Truncate
        if (!label && el.tagName === 'INPUT') label = 'Input';
        if (!label) return;

        // Generate a unique selector or use an internal map key
        // For MVP, we'll attach a data attribute to find it later easily
        const agentId = idCounter++;
        el.setAttribute('data-agent-id', agentId);
        
        items.push({
            id: agentId,
            tag: el.tagName.toLowerCase(),
            type: el.type || '',
            label: label
        });
    });

    return JSON.stringify(items);
})();
