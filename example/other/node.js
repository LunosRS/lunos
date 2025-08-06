import http from 'node:http';

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/plain' });
    res.end('Hello World!');
});

const port = 9595;
server.listen(port, () => {
    console.log(`Server running on http://localhost:${port}`);
});
