document.getElementById('shortenButton').addEventListener('click', async function() {
    const urlInput = document.getElementById('urlInput').value;
    const resultDiv = document.getElementById('result');
    
    if (urlInput === '') {
        resultDiv.innerHTML = '<p>Please enter a URL.</p>';
        return;
    }

    try {
        const response = await fetch('/api', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ url: urlInput })
        });

        const data = await response.json();

        if (response.ok) {
            resultDiv.innerHTML = `<p>Short URL: <a href="${data.short_url	}" target="_blank">${data.short_url	}</a></p>`;
        } else {
            resultDiv.innerHTML = `<p>Error: ${data.error}</p>`;
        }
    } catch (error) {
        resultDiv.innerHTML = `<p>Something went wrong. Please try again later.</p>`;
    }
});
