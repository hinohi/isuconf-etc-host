use anyhow::Context;
use std::{net::IpAddr, str::FromStr};

#[derive(Debug)]
pub struct EtcHosts {
    lines: Vec<Line>,
}

impl EtcHosts {
    pub fn from_str(s: &str) -> anyhow::Result<EtcHosts> {
        let mut lines: Vec<Line> = Vec::new();
        for line in s.lines() {
            lines.push(line.parse()?);
        }
        // remove last empty lines
        while lines.last().iter().any(|line| line.is_empty()) {
            lines.pop();
        }
        Ok(EtcHosts { lines })
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for line in self.lines.iter() {
            line.write_into(&mut s);
            s.push('\n');
        }
        s
    }

    fn add_my_region(&mut self) -> usize {
        for (i, line) in self.lines.iter().enumerate() {
            if let Some(comment) = &line.comment {
                if comment == "ISUCON Servers" {
                    return i;
                }
            }
        }
        let add_empty_line = if let Some(line) = self.lines.last() {
            !line.is_empty()
        } else {
            false
        };
        if add_empty_line {
            self.lines.push(Line {
                data: None,
                comment: None,
            });
        }
        self.lines.push(Line {
            data: None,
            comment: Some("ISUCON Servers".to_owned()),
        });
        self.lines.len() - 1
    }

    pub fn add_data(&mut self, ip: IpAddr, host: &str) {
        remove_first(&mut self.lines, |line| {
            if let Some(data) = &mut line.data {
                if remove_first(&mut data.hosts, |h| h == host) {
                    if data.hosts.is_empty() {
                        return true;
                    }
                }
            }
            false
        });
        let mut i = self.add_my_region();
        while let Some(line) = self.lines.get(i) {
            if line.is_empty() {
                break;
            }
            i += 1;
        }
        self.lines.insert(
            i,
            Line {
                data: Some(LineData {
                    ip,
                    hosts: vec![host.to_owned()],
                }),
                comment: None,
            },
        );
    }
}

fn remove_first<T, F: FnMut(&mut T) -> bool>(v: &mut Vec<T>, mut f: F) -> bool {
    let mut find = None;
    for (i, item) in v.iter_mut().enumerate() {
        if f(item) {
            find = Some(i);
            break;
        }
    }
    if let Some(i) = find {
        v.remove(i);
        true
    } else {
        false
    }
}

#[derive(Debug)]
struct Line {
    data: Option<LineData>,
    comment: Option<String>,
}

impl FromStr for Line {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (stripped, comment) = if let Some(i) = s.find('#') {
            (&s[..i], Some(s[i + 1..].trim_start().to_owned()))
        } else {
            (s, None)
        };
        let data = if stripped.trim().is_empty() {
            None
        } else {
            Some(stripped.parse()?)
        };
        Ok(Line { data, comment })
    }
}

impl Line {
    fn is_empty(&self) -> bool {
        self.data.is_none() && self.comment.is_none()
    }

    fn write_into(&self, buf: &mut String) {
        if let Some(data) = &self.data {
            data.write_into(buf);
        }
        if let Some(comment) = &self.comment {
            if self.data.is_some() {
                buf.push_str("  ");
            }
            buf.push_str("# ");
            buf.push_str(comment);
        }
    }
}

#[derive(Debug)]
struct LineData {
    ip: IpAddr,
    hosts: Vec<String>,
}

impl FromStr for LineData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<LineData, Self::Err> {
        let s = s.trim();
        let space_index = s.find(char::is_whitespace).context("No (ip, host) data")?;
        let ip = s[..space_index].parse()?;
        Ok(LineData {
            ip,
            hosts: s[space_index..]
                .split_whitespace()
                .map(|s| s.to_owned())
                .collect(),
        })
    }
}

impl LineData {
    fn write_into(&self, buf: &mut String) {
        buf.push_str(&self.ip.to_string());
        for host in self.hosts.iter() {
            buf.push(' ');
            buf.push_str(host);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv6Addr;

    #[test]
    fn line_data() {
        let data: LineData = "127.0.0.1 localhost".parse().unwrap();
        assert_eq!(data.ip, IpAddr::from([127, 0, 0, 1]));
        assert_eq!(data.hosts, vec!["localhost".to_owned()]);
    }

    #[test]
    fn line_data_multi_host() {
        let data: LineData = "::1 localhost lh".parse().unwrap();
        assert_eq!(data.ip, IpAddr::from(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
        assert_eq!(data.hosts, vec!["localhost".to_owned(), "lh".to_owned()]);
    }

    #[test]
    fn line_data_no_host() {
        assert!("::1".parse::<LineData>().is_err());
    }

    #[test]
    fn line_empty() {
        let line: Line = " \t  ".parse().unwrap();
        assert!(line.data.is_none());
        assert!(line.comment.is_none());
    }

    #[test]
    fn line_comment() {
        let line: Line = "#  abc".parse().unwrap();
        assert!(line.data.is_none());
        assert_eq!(line.comment, Some("abc".to_owned()));
    }

    #[test]
    fn line() {
        let line: Line = "10.0.0.1 is1  #isucon1  ".parse().unwrap();
        assert_eq!(line.data.unwrap().hosts, vec!["is1".to_owned()]);
        assert_eq!(line.comment, Some("isucon1  ".to_owned()));
    }

    fn etc_hosts() {
        let s = "127.0.0.1 localhost

# The following lines are desirable for IPv6 capable hosts
::1 ip6-localhost ip6-loopback
fe00::0 ip6-localnet

127.0.0.1 is1
10.0.0.2 is2";
    }
}
