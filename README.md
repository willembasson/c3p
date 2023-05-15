# c3p - a copy tool that supports files, s3, scp and URLs (maybe with some compression handling, lets see)
## Work in progress, not usable yet

```
All your copies R mine
🤖　 　　,,''´ ￣ ヽ
　　 　　| |__　 _　|
　 　 　 {{‐'(👁 )Y(👁 )}
  　 　　 !l_l__V^`r'/
　 　　　 ~lr､i_ﾆ_l,'
　　,. r-‐‐]l===l[‐--,r- ､
　 〉､l!　　　｀Y´o　　l!ﾞ,
. //　〉､＿＿L＿＿/il〈.　ﾍ
//　/ }　,'´￣｀ヽ＿{ ﾊV_,ﾍ


  ___  ____  ____
 / __)( __ \(  _ \
( (__  (__ ( ) __/
 \___)(____/(__)

Usage: c3p [OPTIONS] [INPUT] [OUTPUT]
```

### Normal copy
`c3p /home/my_user/file.txt /tmp/file.txt`

### Copy from url to file
`c3p http://www.server.com/path/file.txt /tmp/file.txt`

### Copy from s3 to file (AWS CLI config needs to be set up first)
`c3p s3://source_bucket/folder/file.txt /tmp/file.txt`

### Copy stdin to file
`cat some_file.txt | c3p - /tmp/file.txt`

### Copy from scp to file
`c3p scp://user:pass@server.com:/home/user/.barshrc ~/.bashrc_remote`

### Copy from s3 to s3 (Not implemented yet)
`c3p s3://source_bucket/folder/file.txt s3://target_bucket/folder/file.txt`

### Copy from scp to scp (Not implemented yet)
`c3p me@server.com:~/.barshrc you@other_server.com:~/`

### Copy file to std_out (Not implemented yet)
`c3p /tmp/file.txt - | cat`



