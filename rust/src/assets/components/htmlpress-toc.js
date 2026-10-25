class HtmlpressToc extends HTMLElement {
  connectedCallback() {
    const links = [...this.querySelectorAll("a[href^='#']")]
    const headings = links
      .map(link => document.getElementById(decodeURIComponent(link.hash.slice(1))))
      .filter(Boolean)
    const update = () => {
      // The current heading is the last one scrolled past the header
      const passed = headings.filter(heading => heading.getBoundingClientRect().top < 120)
      const current = passed.at(-1) ?? headings[0]
      for (const link of links) {
        if (link.hash === `#${current?.id}`) link.setAttribute("aria-current", "true")
        else link.removeAttribute("aria-current")
      }
    }
    addEventListener("scroll", update, { passive: true })
    update()
  }
}

customElements.define("htmlpress-toc", HtmlpressToc)
