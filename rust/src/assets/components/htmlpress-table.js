class HtmlpressTable extends HTMLElement {
  connectedCallback() {
    const headers = [...this.querySelectorAll("thead th")]
    headers.forEach((header, column) => {
      header.tabIndex = 0
      const sort = () => this.sort(headers, header, column)
      header.addEventListener("click", sort)
      header.addEventListener("keydown", event => event.key === "Enter" && sort())
    })
  }

  sort(headers, header, column) {
    const ascending = header.getAttribute("aria-sort") !== "ascending"
    headers.forEach(other => other.removeAttribute("aria-sort"))
    header.setAttribute("aria-sort", ascending ? "ascending" : "descending")
    const body = this.querySelector("tbody")
    if (!body) return
    const read = row => row.cells[column]?.textContent.trim() ?? ""
    const rows = [...body.rows].sort((a, b) => {
      const order = read(a).localeCompare(read(b), undefined, { numeric: true })
      return ascending ? order : -order
    })
    body.append(...rows)
  }
}

customElements.define("htmlpress-table", HtmlpressTable)
