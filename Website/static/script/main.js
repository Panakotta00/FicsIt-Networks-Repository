import Asciidoctor from 'https://unpkg.com/@asciidoctor/core/dist/browser/asciidoctor.js'
import { marked } from "https://cdn.jsdelivr.net/npm/marked/lib/marked.esm.js";
const asciidoctor = Asciidoctor()

export function parse_adoc(element) {
    console.info(`Parse Asciidoc:`, element.textContent);
    let html = asciidoctor.convert(element.textContent);
    element.innerHTML = html;
    element.hidden = false;
}

export function parse_md(element) {
    console.info(`Parse Markdown for:`, element.textContent);
    let html = marked.parse(element.textContent);
    element.innerHTML = html;
    element.hidden = false;
}
