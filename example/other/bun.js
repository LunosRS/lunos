Bun.serve({
  port: 9595,
  fetch(req) {
    return new Response("Hello World!");
  },
});

console.log("Server running on http://localhost:9595");
