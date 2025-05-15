Name:           synx
Version:        0.2.0
Release:        1%{?dist}
Summary:        A CLI-first universal syntax validator and linter dispatcher

License:        MIT
URL:            https://github.com/A5873/synx
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rustc
BuildRequires:  gcc
BuildRequires:  make

%description
Synx is a command-line tool that inspects any file and attempts to validate its
syntax or structure by automatically detecting the filetype and dispatching the
appropriate validator or linter. It includes comprehensive memory analysis
capabilities and performance profiling features.

%prep
%autosetup

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
install -D -m 755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}

%changelog
* Thu May 15 2025 Alex Ngugi <ngugialex540@gmail.com> - 0.2.0-1
- Initial RPM release
- Add comprehensive memory analysis capabilities
- Add performance profiling framework
- Enhance validation and detection capabilities
