// Livemark's tables: any table in a page sorts by the column whose header is clicked

function addSorting() {
  const content = document.querySelector("htmlpress-page > main")
  if (!content) return
  for (const table of content.querySelectorAll("table")) {
    const headers = [...table.querySelectorAll(":scope > thead th")]
    const body = table.querySelector(":scope > tbody")
    if (!body || headers.length === 0) continue
    table.classList.add("htmlpress-sortable")
    headers.forEach((header, column) => {
      header.tabIndex = 0
      const sort = () => sortTable(body, headers, header, column)
      header.addEventListener("click", sort)
      header.addEventListener("keydown", event => event.key === "Enter" && sort())
    })
  }
}

function sortTable(body, headers, header, column) {
  const ascending = header.getAttribute("aria-sort") !== "ascending"
  headers.forEach(other => other.removeAttribute("aria-sort"))
  header.setAttribute("aria-sort", ascending ? "ascending" : "descending")
  const read = row => row.cells[column]?.textContent.trim() ?? ""
  const rows = [...body.rows].sort((a, b) => {
    const order = read(a).localeCompare(read(b), undefined, { numeric: true })
    return ascending ? order : -order
  })
  body.append(...rows)
}

addSorting()
