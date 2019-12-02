import module from '../crate/Cargo.toml'

let globalDocument: string;
let globalXMLDocument: Document;
let populateContentTypesDropdown; 
let populateBandwidthDropdown; 

interface IDropdownData {
    value: string,
    text: string,
    name: string,
}

interface IContentType {
    role: string,
    mimeType: string,
}

const initializeComponents = () => {

    // Set up manifest dropdown
    $("#manifest-dropdown").dropdown({
        onChange: onChangeManifestDropdown,
    });

    $("#content-types-dropdown").dropdown({
        onChange: onChangeContentTypesDropdown,
    });

    // Set up button
    $("#parse-button").click(onParseButtonClick);
    $("#media-url-button").click(onMediaUrlButtonClick);
    document.getElementById("manifest-field").addEventListener("paste", (event) => {
        let strToSet = (event as ClipboardEvent).clipboardData.getData('text/plain') || "";
        document.getElementById("manifest-field").setAttribute("value", `${strToSet}`);
    })
    document.getElementById("manifest-field").addEventListener("change", (event) => {
        let currentValue = document.getElementById("manifest-field").getAttribute("value")
        let charToAppend = (event as InputEvent).data || "";
        if ((event as InputEvent).data === null) {
            currentValue = null;
        }
        if (currentValue === null) {
            currentValue = "";
        }
        document.getElementById("manifest-field").setAttribute("value", `${currentValue + charToAppend}`);
    });

    document.getElementById("position-field").addEventListener("input", (event) => {
        let currentValue = document.getElementById("position-field").getAttribute("value")
        let charToAppend = (event as InputEvent).data || "";
        if ((event as InputEvent).data === null) {
            currentValue = null;
        }
        if (currentValue === null) {
            currentValue = "";
        }

        document.getElementById("position-field").setAttribute("value", `${currentValue + charToAppend}`);
    });

    populateContentTypesDropdown = populateDropdown($("#content-types-dropwdown"), onChangeContentTypesDropdown);
    populateBandwidthDropdown = populateDropdown($("#bandwidth-dropwdown"), onChangeBandwidthDropdown);
}

const onChangeManifestDropdown = (value) => {
    document.getElementById("manifest-field").setAttribute("value", value);
}

const onChangeContentTypesDropdown = (contentType) => {
    $("#bandwidth-dropwdown").dropdown("clear");
    populateBandwidthDropdown(getBandwidths(contentType, globalXMLDocument));
}

const onChangeBandwidthDropdown = (value) => {
    // Nothing
}

const onParseButtonClick = () => {
    reset();
    fetchManfifest(document.getElementById("manifest-field").getAttribute("value"));
}

const onMediaUrlButtonClick = () => {
    const position = parseInt($("#position-field").val() as string, 10);
    const bandwidth = parseInt($("#bandwidth-dropwdown").dropdown("get value"), 10);
    const contentType = $("#content-types-dropwdown").dropdown("get value");
    const { mimeType, role } = parseSelectedContentType(contentType)

    const media_url = module.dash_media_url_from_position(globalDocument, position, mimeType, role, bandwidth);
    const message = `Media URL:\n ${media_url}`;
    alert(message);
}

const reset = () => {
    $("#content-types-dropwdown").dropdown("clear");
    $("#bandwidth-dropwdown").dropdown("clear");
}

const fetchManfifest = (url: string) => {
    fetch(url)
    .then(response => response.text())
    .then(str => {
        globalDocument = str;
        globalXMLDocument = (new window.DOMParser()).parseFromString(str, "text/xml");
        populateContentTypesDropdown(getContentTypes(globalXMLDocument));
        $("#content-types-container").show();
    })
}

const getContentTypes = (document: Document): IDropdownData[] => {
    return Array.from(new Set(Array.from(document.querySelectorAll("AdaptationSet"))
                                .map(extractContentTypeFromASet)))
                                .map(generateDropdownData);
    
}

const getBandwidths = (contentType: string, xml: Document): IDropdownData[] => {
    const bandwidthSet = new Set(Array.from(xml.querySelectorAll("AdaptationSet"))
        .filter(aset => extractContentTypeFromASet(aset) === contentType)
        .map(aset => Array.from(aset.querySelectorAll("Representation")))
        .reduce((acc, val) => acc.concat(val), [])
        .map(rep => rep.getAttribute("bandwidth")));

    return Array.from(bandwidthSet)
            .map(generateDropdownData);
}

const populateDropdown = (dropdown: JQuery<HTMLElement>, onChangeHandler) => (values: IDropdownData[]) => {
    dropdown.dropdown({
        onChange: onChangeHandler,
    });
    dropdown.dropdown("setup menu", { values });
}

const generateDropdownData = (data: string): IDropdownData => {
    return { value: data, text: data, name: data}
}

const extractContentTypeFromASet = (aset: Element) => {
    let role = "main";
    let mimeType: string = aset.getAttribute("mimeType");
    if (aset.querySelector("Role")) {
        role = aset.querySelector("Role").getAttribute("value");
    }
    return `${role}-${mimeType}`;
}

const parseSelectedContentType = (contentType: string): IContentType => {
    const splitContentType = contentType.split("-");
    return {
        role: splitContentType[0],
        mimeType: splitContentType[1],
    }
}

initializeComponents();
