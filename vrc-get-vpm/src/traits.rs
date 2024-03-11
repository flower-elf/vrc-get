use crate::io;
use crate::io::EnvironmentIo;
use crate::repository::local::LocalCachedRepository;
use crate::utils::MapResultExt;
use crate::{PackageInfo, PackageJson, VersionSelector};
use core::iter::Iterator;
use core::option::Option;
use futures::prelude::*;
use indexmap::IndexMap;
use std::convert::Infallible;
use url::Url;

pub trait PackageCollection {
    /// get curated packages
    fn get_curated_packages(
        &self,
        _version_selector: VersionSelector,
    ) -> impl Iterator<Item = PackageInfo> {
        [].into_iter()
    }

    /// get all packages in the collection
    fn get_all_packages(&self) -> impl Iterator<Item = PackageInfo>;

    /// get all package versions of the specified package
    fn find_packages(&self, package: &str) -> impl Iterator<Item = PackageInfo>;

    /// get specified version of specified package
    fn find_package_by_name(
        &self,
        package: &str,
        package_selector: VersionSelector,
    ) -> Option<PackageInfo>;
}

pub trait EnvironmentIoHolder {
    type EnvironmentIo: EnvironmentIo;
    fn io(&self) -> &Self::EnvironmentIo;
}

/// The trait for downloading remote packages.
///
/// Caching packages is responsibility of this crate.
pub trait RemotePackageDownloader {
    type FileStream: AsyncRead + AsyncSeek + Unpin;

    /// Get package from remote server.
    fn get_package(
        &self,
        repository: &LocalCachedRepository,
        package: &PackageJson,
    ) -> impl Future<Output = io::Result<Self::FileStream>> + Send;
}

/// The HTTP Client.
pub trait HttpClient: Sync {
    /// Get resource from the URL with specified headers
    ///
    /// Note: If remote server returns error status code, this function should return error.
    fn get(
        &self,
        url: &Url,
        headers: &IndexMap<Box<str>, Box<str>>,
    ) -> impl Future<Output = io::Result<impl AsyncRead + Send>> + Send;

    /// Get resource from the URL with specified headers and etag
    ///
    /// Returning `Ok(None)` means cache matched.
    /// Returning `Ok(Some((stream, etag)))` means cache not matched and get from remote server.
    /// Returning `Err(_)` means error.
    ///
    /// Note: If remote server returns error status code, this function should return error.
    fn get_with_etag(
        &self,
        url: &Url,
        headers: &IndexMap<Box<str>, Box<str>>,
        current_etag: Option<&str>,
    ) -> impl Future<Output = io::Result<Option<(impl AsyncRead + Send, Option<Box<str>>)>>> + Send;
}

impl HttpClient for reqwest::Client {
    async fn get(
        &self,
        url: &Url,
        headers: &IndexMap<Box<str>, Box<str>>,
    ) -> io::Result<impl AsyncRead> {
        // file not found: err

        let mut request = self.get(url.to_owned());

        for (name, header) in headers {
            request = request.header(name.as_ref(), header.as_ref());
        }

        Ok(request
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .err_mapped()?
            .bytes_stream()
            .map(|x| x.err_mapped())
            .into_async_read())
    }

    async fn get_with_etag(
        &self,
        url: &Url,
        headers: &IndexMap<Box<str>, Box<str>>,
        current_etag: Option<&str>,
    ) -> io::Result<Option<(impl AsyncRead, Option<Box<str>>)>> {
        let mut request = self.get(url.to_owned());
        for (name, value) in headers {
            request = request.header(name.as_ref(), value.as_ref());
        }
        if let Some(etag) = current_etag {
            request = request.header("If-None-Match", etag.to_owned())
        }
        let response = request.send().await.err_mapped()?;
        let response = response.error_for_status().err_mapped()?;

        if current_etag.is_some() && response.status() == 304 {
            // for requests with etag, 304 means cache matched
            return Ok(None);
        }

        let etag = response
            .headers()
            .get("Etag")
            .and_then(|x| x.to_str().ok())
            .map(Into::into);

        // response.json() doesn't support BOM
        let response_stream = response
            .bytes_stream()
            .map(|x| x.err_mapped())
            .into_async_read();

        Ok(Some((response_stream, etag)))
    }
}

impl HttpClient for Infallible {
    async fn get(&self, _: &Url, _: &IndexMap<Box<str>, Box<str>>) -> io::Result<impl AsyncRead> {
        Ok(io::empty())
    }

    async fn get_with_etag(
        &self,
        _: &Url,
        _: &IndexMap<Box<str>, Box<str>>,
        _: Option<&str>,
    ) -> io::Result<Option<(impl AsyncRead, Option<Box<str>>)>> {
        Ok(Some((io::empty(), None)))
    }
}
