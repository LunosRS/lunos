console.log("Server running on http://localhost:9595");

Deno.serve((req) => new Response("Hello World!"), { port: 9595 });

