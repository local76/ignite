Name:           rstart
Version:        TEMPLATE_VERSION
Release:        1%{?dist}
Summary:        A project template for creating unified local terminal utilities in Rust
License:        MIT
URL:            https://github.com/local76/rStartrt
Source0:        %{name}-%{version}.tar.gz

%description
A project template for creating unified local terminal utilities in Rust.

%prep
%setup -q

%build
cargo build --release --locked

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT/%{_bindir}
install -d $RPM_BUILD_ROOT/%{_datadir}/applications
install -d $RPM_BUILD_ROOT/%{_datadir}/pixmaps
install -m 755 target/release/rstart $RPM_BUILD_ROOT/%{_bindir}/rstart
install -m 644 packaging/desktop/rstart.desktop $RPM_BUILD_ROOT/%{_datadir}/applications/rstart.desktop
install -m 644 assets/brand/app_icon.png $RPM_BUILD_ROOT/%{_datadir}/pixmaps/rstart.png

%files
%{_bindir}/rstart
%{_datadir}/applications/rstart.desktop
%{_datadir}/pixmaps/rstart.png
