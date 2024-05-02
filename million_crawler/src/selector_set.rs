use scraper::{Html, Selector};
use url::Url;

pub struct SelectorSet {
    href_selector: Selector,
    codebase_selector: Selector,
    cite_selector: Selector,
    background_selector: Selector,
    action_selector: Selector,
    longdesc_selector: Selector,
    src_selector: Selector,
    profile_selector: Selector,
    usemap_selector: Selector,
    classid_selector: Selector,
    data_selector: Selector,
    formaction_selector: Selector,
    icon_selector: Selector,
    manifest_selector: Selector,
    poster_selector: Selector,

    srcset_selector: Selector,
    archive_selector: Selector,
    meta_http_equiv_refresh_content_selector: Selector,
}

impl SelectorSet {
    pub fn new() -> Self {
        Self {
            href_selector: Selector::parse("[href]").unwrap(),
            codebase_selector: Selector::parse("[codebase]").unwrap(),
            cite_selector: Selector::parse("[cite]").unwrap(),
            background_selector: Selector::parse("[background]").unwrap(),
            action_selector: Selector::parse("[action]").unwrap(),
            longdesc_selector: Selector::parse("[longdesc]").unwrap(),
            src_selector: Selector::parse("[src]").unwrap(),
            profile_selector: Selector::parse("[profile]").unwrap(),
            usemap_selector: Selector::parse("[usemap]").unwrap(),
            classid_selector: Selector::parse("[classid]").unwrap(),
            data_selector: Selector::parse("[data]").unwrap(),
            formaction_selector: Selector::parse("[formaction]").unwrap(),
            icon_selector: Selector::parse("[icon]").unwrap(),
            manifest_selector: Selector::parse("[manifest]").unwrap(),
            poster_selector: Selector::parse("[poster]").unwrap(),
            srcset_selector: Selector::parse("[srcset]").unwrap(),
            archive_selector: Selector::parse("[archive]").unwrap(),
            meta_http_equiv_refresh_content_selector: Selector::parse(
                "meta[http-equiv=\"refresh\"][content]",
            )
            .unwrap(),
        }
    }

    pub fn select(&self, doc: &Html, page_url: &Url) -> Vec<Url> {
        let href_tags = doc
            .select(&self.href_selector)
            .map(|elem| elem.attr("href").unwrap());
        let codebase_tags = doc
            .select(&self.codebase_selector)
            .map(|elem| elem.attr("codebase").unwrap());
        let cite_tags = doc
            .select(&self.cite_selector)
            .map(|elem| elem.attr("cite").unwrap());
        let background_tags = doc
            .select(&self.background_selector)
            .map(|elem| elem.attr("background").unwrap());
        let action_tags = doc
            .select(&self.action_selector)
            .map(|elem| elem.attr("action").unwrap());
        let longdesc_tags = doc
            .select(&self.longdesc_selector)
            .map(|elem| elem.attr("longdesc").unwrap());
        let src_tags = doc
            .select(&self.src_selector)
            .map(|elem| elem.attr("src").unwrap());
        let profile_tags = doc
            .select(&self.profile_selector)
            .map(|elem| elem.attr("profile").unwrap());
        let usemap_tags = doc
            .select(&self.usemap_selector)
            .map(|elem| elem.attr("usemap").unwrap());
        let classid_tags = doc
            .select(&self.classid_selector)
            .map(|elem| elem.attr("classid").unwrap());
        let data_tags = doc
            .select(&self.data_selector)
            .map(|elem| elem.attr("data").unwrap());
        let formaction_tags = doc
            .select(&self.formaction_selector)
            .map(|elem| elem.attr("formaction").unwrap());
        let icon_tags = doc
            .select(&self.icon_selector)
            .map(|elem| elem.attr("icon").unwrap());
        let manifest_tags = doc
            .select(&self.manifest_selector)
            .map(|elem| elem.attr("manifest").unwrap());
        let poster_tags = doc
            .select(&self.poster_selector)
            .map(|elem| elem.attr("poster").unwrap());

        let srcset_tags = doc
            .select(&self.srcset_selector)
            .map(|elem| elem.attr("srcset").unwrap())
            .flat_map(|attr| attr.split(',').map(|csl| csl.split(' ').nth(1)))
            .filter_map(|url| url);
        let archive_tags = doc
            .select(&self.archive_selector)
            .map(|elem| elem.attr("archive").unwrap())
            .flat_map(|attr| attr.split_whitespace().flat_map(|attr| attr.split(',')));
        let meta_http_equiv_refresh_content_tags = doc
            .select(&self.meta_http_equiv_refresh_content_selector)
            .map(|elem| elem.attr("content").unwrap())
            .filter_map(|attr| attr.split(';').nth(1));

        let tags = href_tags
            .chain(codebase_tags)
            .chain(cite_tags)
            .chain(background_tags)
            .chain(action_tags)
            .chain(longdesc_tags)
            .chain(src_tags)
            .chain(profile_tags)
            .chain(usemap_tags)
            .chain(classid_tags)
            .chain(data_tags)
            .chain(formaction_tags)
            .chain(icon_tags)
            .chain(manifest_tags)
            .chain(poster_tags)
            .chain(srcset_tags)
            .chain(archive_tags)
            .chain(meta_http_equiv_refresh_content_tags)
            .filter_map(|url| Self::normalize_url(url, page_url).ok());

        tags.collect()
    }

    fn normalize_url(url: &str, base_url: &Url) -> anyhow::Result<Url> {
        Ok(base_url.join(url)?)
    }
}
