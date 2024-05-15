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

    title_selector: Selector,
    description_selector: Selector,
    icon_link_selector: Selector,

    text_fields_selector: Selector,
    sections_selector: Selector,

    link_manifest_selector: Selector,

    images_selector: Selector,
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

            title_selector: Selector::parse("title").unwrap(),
            description_selector: Selector::parse("meta[name=\"description\"][content]").unwrap(),
            icon_link_selector: Selector::parse("link[rel=\"icon\"][href]").unwrap(),

            text_fields_selector: Selector::parse("p").unwrap(),
            sections_selector: Selector::parse("h1, h2, h3, h4, h5, h6").unwrap(),

            link_manifest_selector: Selector::parse("link[rel=\"manifest\"][href]").unwrap(),

            images_selector: Selector::parse("img[src]").unwrap(),
        }
    }

    pub fn select_manifest_url(&self, doc: &Html, page_url: &Url) -> Option<Url> {
        doc.select(&self.link_manifest_selector)
            .next()
            .map(|link_manifest| link_manifest.attr("href"))
            .flatten()
            .map(|link_manifest_url| Self::normalize_url(link_manifest_url, page_url).ok())
            .flatten()
    }

    pub fn select_images(&self, doc: &Html, page_url: &Url) -> Vec<(Url, Option<String>)> {
        doc.select(&self.images_selector)
            .map(|image| (image.attr("src").unwrap(), image.attr("alt")))
            .filter_map(|(image_url, image_alt_text)| {
                Some((
                    Self::normalize_url(image_url, page_url).ok()?,
                    image_alt_text.map(|iat| iat.to_owned()),
                ))
            })
            .collect()
    }

    pub fn select_title(&self, doc: &Html) -> Option<String> {
        doc.select(&self.title_selector)
            .next()
            .map(|title| title.text().map(|text| text.to_string()).collect())
    }

    pub fn select_description(&self, doc: &Html) -> Option<String> {
        doc.select(&self.description_selector)
            .next()
            .map(|description| description.attr("content"))
            .flatten()
            .map(|description| description.to_owned())
    }

    pub fn select_icon_url(&self, doc: &Html, page_url: &Url) -> Option<Url> {
        doc.select(&self.icon_link_selector)
            .next()
            .map(|icon_url| icon_url.attr("href"))
            .flatten()
            .map(|icon_url| Self::normalize_url(icon_url, page_url).ok())
            .flatten()
    }

    pub fn select_text_fields(&self, doc: &Html) -> Vec<String> {
        doc.select(&self.text_fields_selector)
            .map(|text| text.text().map(|text| text.to_owned()).collect())
            .collect()
    }

    pub fn select_sections(&self, doc: &Html) -> Vec<String> {
        doc.select(&self.sections_selector)
            .map(|sections| {
                sections
                    .text()
                    .map(|sections| sections.to_owned())
                    .collect()
            })
            .collect()
    }

    pub fn select_urls(&self, doc: &Html, page_url: &Url) -> Vec<Url> {
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
