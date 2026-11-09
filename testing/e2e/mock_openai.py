"""A stand-in for the OpenAI Responses API, for `make e2e-oagw`.

Answers every POST with one fixed receipt and appends the request path, model
and Authorization header to the log file given as the second argument.

    python3 mock_openai.py <port> <log file>
"""
import json
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer

LOG = sys.argv[2]
RECEIPT = {
    "merchant": "Mock Market", "date": "2026-11-08", "time": "10:15", "currency": "EUR",
    "total_minor": 449, "adjustments_minor": 0, "restaurant": False, "needs_review": False,
    "issues": [],
    "items": [{"name": "Cat food", "translated_name": "Cat food 6 pouches", "raw_name": "CAT FOOD 6X",
               "product_id": None, "variant": "", "category": "pets", "subcategory": "Pet food",
               "quantity": 1, "unit": "pc", "amount_minor": 449, "uncertain": False}],
}


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("content-length", "0"))
        body = json.loads(self.rfile.read(length) or b"{}")
        with open(LOG, "a") as log:
            log.write(json.dumps({"path": self.path, "authorization": self.headers.get("authorization"),
                                  "model": body.get("model")}) + "\n")
        answer = {"status": "completed",
                  "output": [{"content": [{"type": "output_text", "text": json.dumps(RECEIPT)}]}]}
        data = json.dumps(answer).encode()
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *args):
        pass


HTTPServer(("127.0.0.1", int(sys.argv[1])), Handler).serve_forever()
